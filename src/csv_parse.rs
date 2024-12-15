use std::collections::HashMap;
use itertools::Itertools;


#[derive(Debug, Clone)]
pub struct Nutrient {
    name: String,
    display_name: String,
    abbreviation: String,
    units: String,
    recommended_intake: f32,
}


#[derive(Debug, Clone)]
pub struct Food {
    pub name: String,
    nutrients: HashMap<String, f32>,
}

fn make_food(
    record: csv::StringRecord,
    nutrients: &Vec<Nutrient>,
) -> Food {
    let name = record
        .get(0)
        .expect("each row has at least 1 record")
        .to_owned();
    let nutrient_values = std::iter::zip(
        nutrients.iter(),
        record.iter().skip(1)
    )
        .map(|(n,x)| match x.parse::<f32>() {
            Ok(f) => (
                n.name.to_string(),
                f,
            ),
            Err(_) => (
                n.name.to_string(),
                n.recommended_intake / 5.,
            ),
        })
        .collect::<HashMap<String, f32>>();
    Food {
        name: name,
        nutrients: nutrient_values,
    }
}

fn get_nutrients(
    reader: &mut csv::Reader<&[u8]>
) -> Vec<Nutrient> {
    let mut nutrients = Vec::<Nutrient>::new();
    let mut headers: Vec<Vec<String>> = reader
        .records()
        .take(5)
        .map(|r| r
            .expect("cofid.csv is error free")
            .into_iter()
            .skip(1)
            .map(|s| s.to_owned())
            .collect()
        )
        .collect();
    for _ in 0..headers[0].len() {
        let recommended_intake: f32 = headers[4]
            .remove(0)
            .parse()
            .map_or(
                0.,
                |s| s
            );
        let new_nutrient = Nutrient {
            name: headers[0].remove(0),
            display_name: headers[3].remove(0),
            abbreviation: headers[2].remove(0),
            units: headers[1].remove(0),
            recommended_intake: recommended_intake,
        };
        nutrients.push(new_nutrient);
    }
    return nutrients;
}

pub fn get_foods(csv: String) -> (Vec<Nutrient>, Vec<Food>) {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(csv.as_bytes());

    let nutrients = get_nutrients(&mut reader);
    let foods = reader
        .records()
        .map(|r| make_food(
            r.expect("cofid.csv is error free"),
            &nutrients,
        ))
        .collect::<Vec<Food>>();

    (nutrients, foods)
}

fn match_score(food: &Food, search_words: &Vec<String>) -> usize {
    search_words
        .into_iter()
        .fold(0, |a, s| 
            a + (food.name.to_lowercase().contains(s) as usize)
        ) * 1000 + 1000 / food.name.len()
}

fn lookup_food(
    foods: &Vec<Food>, search: String
) -> Option<&Food> {
    let search_words = search
        .split(" ")
        .map(|s| s.trim().to_lowercase())
        .collect::<Vec<String>>();
    let best_match = foods
        .iter()
        .max_by_key(|f| match_score(f, &search_words))
        .expect("foods is nonempty");
    if match_score(best_match, &search_words) > 1000 {
        Some(best_match)
    } else {
        None
    }
}

pub fn lookup_foods(
    foods: &Vec<Food>, search: String
) -> Vec<&Food> {
    let searches = search
        .split(",")
        .map(|s| s.trim().to_string());
    searches
        .filter_map(|s| lookup_food(foods, s))
        .collect::<Vec<&Food>>()
}

pub fn sum_nutrients(
    nutrients: &Vec<Nutrient>, foods: &Vec<&Food>
) -> HashMap<String, f32> {
    nutrients
        .iter()
        .map(|n| (
            n.name.clone(),
            foods
                .iter()
                .fold(
                    0., 
                    |a, f| a + f.nutrients[&n.name],
                )
            )
        )
        .collect::<HashMap<String, f32>>()
}

fn balance_score(
    food: &Food, ideal_nutrients: &HashMap<String, f32>
) -> usize {
    ideal_nutrients
        .iter()
        .fold(0, |a, (s, _)|
            a + std::cmp::min(
                (1000. * food.nutrients[s] / ideal_nutrients[s])
                    as usize,
                1000
            ),
        )
}

pub fn recommend_foods<'a>(
    nutrients: &Vec<Nutrient>,
    foods: &'a Vec<Food>,
    nutrients_sum: &HashMap<String, f32>,
) -> Vec<&'a Food> {
    let ideal_nutrients = nutrients
        .iter()
        .map(|n| (
            n.name.clone(), 
            n.recommended_intake - nutrients_sum[&n.name],
        ))
        .collect::<HashMap<String, f32>>();
    foods
        .iter()
        .k_largest_by_key(
            3,
            |f| balance_score(&f, &ideal_nutrients)
        )
        .collect::<Vec<&Food>>()
}

fn main() -> () {}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_foods() -> (Vec<Nutrient>, Vec<Food>) {
        let csv = std::fs::read_to_string(
            "./assets/cofid.csv"
        ).expect("cofid.csv is error free");
        super::get_foods(csv)
    }

    #[test]
    fn csv_parses_ok() -> () {
        let (_nutrients, foods) = get_foods();
        assert_eq!(foods.len(), 1554);
        assert_eq!(foods[0].nutrients.len(), 58);
    }

    #[test]
    fn search_food() -> () {
        let (_nutrients, foods) = get_foods();
        let found_food = lookup_food(&foods, "Ackee".to_string())
            .expect("should find a food");
        assert_eq!(found_food.name, "Ackee, canned, drained");
        assert_eq!(found_food.nutrients["vitamin_c_mg"], 30.0);
    }

    #[test]
    fn search_food_multi_word() -> () {
        let (_nutrients, foods) = get_foods();
        let found_food = lookup_food(
            &foods,
            "rice pudding".to_string()
        ).expect("should find a food");
        assert_eq!(
            found_food.name,
            "Pudding, rice, canned"
        );
        let found_food2 = lookup_food(
            &foods,
            "apple baked sugar".to_string()
        ).expect("should find a food");
        assert_eq!(
            found_food2.name,
            "Apples, cooking, baked with sugar, flesh only"
        );
    }

    #[test]
    fn search_foods() -> () {
        let (_nutrients, foods) = get_foods();
        let found_foods = lookup_foods(
            &foods,
            "Ackee, Amla, Apples".to_string()
        );
        assert_eq!(found_foods[0].name, "Ackee, canned, drained");
        assert_eq!(found_foods[1].name, "Amla");
        assert_eq!(found_foods[2].name, "Apples, eating, dried");

        let found_foods2 = lookup_foods(
            &foods,
            "Ackee, Amla, baked apple".to_string()
        );
        assert_eq!(found_foods2[0].name, "Ackee, canned, drained");
        assert_eq!(found_foods2[1].name, "Amla");
        assert_eq!(
            found_foods2[2].name,
            "Apples, cooking, baked with sugar, flesh only"
        );
    }

    #[test]
    fn search_foods_without_match() -> () {
        let (_nutrients, foods) = get_foods();
        let lookup_result = lookup_food(
            &foods,
            "glorb".to_string()
        );
        assert!(lookup_result.is_none());

        let found_foods = lookup_foods(
            &foods,
            "Ackee, glorb, baked apple".to_string()
        );
        assert_eq!(found_foods[0].name, "Ackee, canned, drained");
        assert_eq!(
            found_foods[1].name,
            "Apples, cooking, baked with sugar, flesh only"
        );
    }

    #[test]
    fn sum_nutrients() -> () {
        let (nutrients, foods) = get_foods();
        let found_foods = lookup_foods(
            &foods,
            "Ackee, Amla, Apples".to_string()
        );
        let nutrients_sum = super::sum_nutrients(
            &nutrients,
            &found_foods
        );
        assert_eq!(nutrients_sum["vitamin_c_mg"], 38.);
        assert_eq!(nutrients_sum["vitamin_b12_ug"], 0.);
    }

    #[test]
    fn recommend() -> () {
        let (nutrients, foods) = get_foods();
        let found_foods = lookup_foods(
            &foods,
            "Ackee, Amla, Apples".to_string()
        );
        let nutrients_sum = super::sum_nutrients(
            &nutrients,
            &found_foods
        );
        let recommended_foods = recommend_foods(
            &nutrients,
            &foods,
            &nutrients_sum
        );
        assert_eq!(
            recommended_foods[0].name,
            "Breakfast cereal, bran type cereal, fortified"
        );
        assert_eq!(
            recommended_foods[1].name,
            "Breakfast cereal, instant hot oat, plain, raw, fortified"
        );
        assert_eq!(
            recommended_foods[2].name,
            "Wheatgerm"
        );
    }
}
