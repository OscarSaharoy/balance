use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum NutrientValue {
    Code(String),
    Value(f32),
}

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
    name: String,
    nutrients: HashMap<String, NutrientValue>,
}

fn make_food(
    record: csv::StringRecord,
    mut nutrient_names: Vec<String>,
) -> Food {
    let name = record
        .get(0)
        .expect("each row has at least 1 record")
        .to_owned();
    let values = record
        .iter()
        .map(|x| match x.parse::<f32>() {
            Ok(f) => NutrientValue::Value(f),
            Err(_) => NutrientValue::Code(x.to_owned()),
        });
    Food {
        name: name,
        nutrients: values
            .map(|x| (nutrient_names.remove(0), x))
            .collect::<HashMap<String, NutrientValue>>(),
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
            .map(|s| s.to_owned())
            .collect()
        )
        .collect();
    for _ in 0..headers[0].len() {
        let recommended_intake: f32 = headers[4]
            .remove(0)
            .parse()
            .map_or(0., |s| s);
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

pub fn get_foods(csv: String) -> Vec<Food> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(csv.as_bytes());

    let nutrients = get_nutrients(&mut reader);
    let nutrient_names = nutrients
        .iter()
        .map(|n| n.name.clone())
        .collect::<Vec<String>>();

    reader
        .records()
        .map(|r| make_food(
            r.expect("cofid.csv is error free"),
            nutrient_names.clone(),
        ))
        .collect::<Vec<Food>>()
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
    foods
        .iter()
        .filter(|f| match_score(f, &search_words) > 1000)
        .max_by_key(|f| match_score(f, &search_words))
}

fn lookup_foods(
    foods: &Vec<Food>, search: String
) -> Vec<&Food> {
    let searches = search
        .split(",")
        .map(|s| s.trim().to_string());
    searches
        .filter_map(|s| lookup_food(foods, s))
        .collect::<Vec<&Food>>()
}

/*
fn sum_nutrients(foods: Vec<Food>)
    -> HashMap<String, NutrientValue> {
}
*/

#[cfg(test)]
mod tests {
    use super::*;

    fn get_foods() -> Vec<Food> {
        let csv = std::fs::read_to_string(
            "./assets/cofid.csv"
        ).expect("cofid.csv is error free");
        super::get_foods(csv)
    }

    #[test]
    fn csv_parses_ok() -> () {
        let foods = get_foods();
        assert_eq!(foods.len(), 2887);
        assert_eq!(foods[0].nutrients.len(), 59);
    }

    #[test]
    fn search_food() -> () {
        let foods = get_foods();
        let found_food = lookup_food(&foods, "Ackee".to_string())
            .expect("should find a food");
        assert_eq!(found_food.name, "Ackee, canned, drained");
        assert_eq!(
            found_food.nutrients["vitamin_c_mg"],
            NutrientValue::Value(30.0)
        );
    }

    #[test]
    fn search_food_multi_word() -> () {
        let foods = get_foods();
        let found_food = lookup_food(
            &foods,
            "Yorkshire pudding milk".to_string()
        ).expect("should find a food");
        assert_eq!(
            found_food.name,
            "Yorkshire pudding, made with whole milk"
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
        let foods = get_foods();
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
        let foods = get_foods();
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
}
