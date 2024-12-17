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
    recommend: bool,
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
    let recommend = match record
        .get(1)
        .expect("each row has at least 2 records") {
        "TRUE" => true,
        _ => false,
    };
    let nutrient_values = std::iter::zip(
        nutrients.iter(),
        record.iter().skip(2)
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
        recommend: recommend,
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
            .skip(2)
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
        .map(|s| s.trim().to_string())
        .filter(|s| s.len() > 0);
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
    nutrients: &Vec<Nutrient>,
    food: &Food,
    nutrients_sum: &HashMap<String, f32>
) -> i64 {
    nutrients
        .iter()
        .filter(|n| n.recommended_intake > 0.1)
        .map(|n| ((( 1000. * food.nutrients[&n.name] / n.recommended_intake ) * ( 1. - 4. * nutrients_sum[&n.name] / n.recommended_intake )) as i64).min(1000).max(-1000))
        .sum()
}

pub fn recommend_foods<'a>(
    nutrients: &Vec<Nutrient>,
    foods: &'a Vec<Food>,
    nutrients_sum: &HashMap<String, f32>,
) -> Vec<&'a Food> {
    foods
        .iter()
        .filter(|f| f.recommend)
        .k_largest_by_key(
            3,
            |f| balance_score(nutrients, &f, &nutrients_sum)
        )
        .collect::<Vec<&Food>>()
}

pub fn get_highest_and_lowest_nutrients<'a>(
    nutrients: &'a Vec<Nutrient>,
    nutrient_values: &HashMap<String, f32>
) -> (&'a Nutrient, &'a Nutrient) {
    let rank_nutrient = move |n: &&Nutrient| ( nutrient_values[&n.name] / n.recommended_intake * 1000. ) as usize;
    (
        nutrients
            .iter()
            .filter(|n| n.recommended_intake > 0.1)
            .max_by_key(rank_nutrient)
            .expect("nutrients is nonempty"),
        nutrients
            .iter()
            .filter(|n| n.recommended_intake > 0.1)
            .min_by_key(rank_nutrient)
            .expect("nutrients is nonempty"),
    )
}

fn main() -> () {}

#[cfg(test)]
mod tests {
    fn get_foods() -> (Vec<super::Nutrient>, Vec<super::Food>) {
        let csv = std::fs::read_to_string(
            "./assets/cofid.csv"
        ).expect("cofid.csv is error free");
        super::get_foods(csv)
    }

    #[test]
    fn csv_parses_ok() -> () {
        let (_nutrients, foods) = get_foods();
        assert_eq!(foods.len(), 2887);
        assert_eq!(foods[0].nutrients.len(), 58);
    }

    #[test]
    fn search_food() -> () {
        let (_nutrients, foods) = get_foods();
        let found_food = super::lookup_food(
            &foods,
            "Ackee".to_string()
        ).expect("should find a food");
        assert_eq!(found_food.name, "Ackee, canned, drained");
        assert_eq!(found_food.nutrients["vitamin_c_mg"], 30.0);
    }

    #[test]
    fn search_food_multi_word() -> () {
        let (_nutrients, foods) = get_foods();
        let found_food = super::lookup_food(
            &foods,
            "rice pudding".to_string()
        ).expect("should find a food");
        assert_eq!(
            found_food.name,
            "Pudding, rice, canned"
        );
        let found_food2 = super::lookup_food(
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
        let found_foods = super::lookup_foods(
            &foods,
            "Ackee, Amla, Apples".to_string()
        );
        assert_eq!(found_foods[0].name, "Ackee, canned, drained");
        assert_eq!(found_foods[1].name, "Amla");
        assert_eq!(found_foods[2].name, "Apples, eating, dried");

        let found_foods2 = super::lookup_foods(
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
        let lookup_result = super::lookup_food(
            &foods,
            "glorb".to_string()
        );
        assert!(lookup_result.is_none());

        let found_foods = super::lookup_foods(
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
        let found_foods = super::lookup_foods(
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
        let found_foods = super::lookup_foods(
            &foods,
            "Ackee, Amla, Apples".to_string()
        );
        let nutrients_sum = super::sum_nutrients(
            &nutrients,
            &found_foods
        );
        let recommended_foods = super::recommend_foods(
            &nutrients,
            &foods,
            &nutrients_sum
        );
        assert_eq!(
            recommended_foods[0].name,
            "Wheatgerm"
        );
        assert_eq!(
            recommended_foods[1].name,
            "Flour, soya"
        );
        assert_eq!(
            recommended_foods[2].name,
            "Bran, wheat"
        );
    }

    #[test]
    fn highest_and_lowest_nutrients() -> () {
        let (nutrients, foods) = get_foods();
        let ackee = &foods[0];
        assert_eq!(ackee.name, "Ackee, canned, drained");
        let (highest_nutrient, lowest_nutrient) = 
            super::get_highest_and_lowest_nutrients(
                &nutrients, &ackee.nutrients
            );
        assert_eq!(highest_nutrient.name, "vitamin_c_mg");
        assert_eq!(lowest_nutrient.name, "fibre_g");

        let found_foods = super::lookup_foods(
            &foods,
            "Yeast, Yeast, Yeast".to_string()
        );
        let nutrients_sum = super::sum_nutrients(
            &nutrients,
            &found_foods
        );
        let (highest_nutrient, lowest_nutrient) = 
            super::get_highest_and_lowest_nutrients(
                &nutrients, &nutrients_sum
            );
        assert_eq!(highest_nutrient.name, "folate_ug");
        assert_eq!(lowest_nutrient.name, "retinol_ug");
    }
}
