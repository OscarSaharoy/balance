use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum NutrientValue {
    Code(String),
    Value(f32),
}

impl<'a, 'b> std::ops::Add<&'a NutrientValue> for &'b NutrientValue {
    type Output = NutrientValue;
    fn add(self, other: &'a NutrientValue) -> NutrientValue {
        if let NutrientValue::Code(s) = self {
            NutrientValue::Code(s.to_string())
        }
        else if let NutrientValue::Code(s) = other {
            NutrientValue::Code(s.to_string())
        }
        else if let (NutrientValue::Value(v1), NutrientValue::Value(v2)) = (self, other) {
            NutrientValue::Value(v1 + v2)
        }
        else {
            NutrientValue::Value(0.0)
        }
    }
}

impl<'a, 'b> std::ops::Sub<&'a NutrientValue> for &'b NutrientValue {
    type Output = NutrientValue;
    fn sub(self, other: &'a NutrientValue) -> NutrientValue {
        if let NutrientValue::Code(s) = other {
            NutrientValue::Value(0.)
        }
        else if let NutrientValue::Code(s) = self {
            NutrientValue::Code(s.to_string())
        }
        else if let (NutrientValue::Value(v1), NutrientValue::Value(v2)) = (self, other) {
            NutrientValue::Value(v1 - v2)
        }
        else {
            NutrientValue::Value(0.0)
        }
    }
}

#[derive(Debug, Clone)]
pub struct Nutrient {
    name: String,
    display_name: String,
    abbreviation: String,
    units: String,
    recommended_intake: NutrientValue,
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
        let recommended_intake: NutrientValue = headers[4]
            .remove(0)
            .parse()
            .map_or(
                NutrientValue::Value(0.),
                |s| NutrientValue::Value(s)
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
    let nutrient_names = nutrients
        .iter()
        .map(|n| n.name.clone())
        .collect::<Vec<String>>();

    let foods = reader
        .records()
        .map(|r| make_food(
            r.expect("cofid.csv is error free"),
            nutrient_names.clone(),
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

fn sum_nutrients(
    nutrients: &Vec<Nutrient>, foods: &Vec<&Food>
) -> HashMap<String, NutrientValue> {
    nutrients
        .iter()
        .map(|n| (
            n.name.clone(),
            foods
                .iter()
                .fold(
                    NutrientValue::Value(0.), 
                    |a, f| &a + &f.nutrients[&n.name],
                )
            )
        )
        .collect::<HashMap<String, NutrientValue>>()
}

fn recommend_foods(
    nutrients: &Vec<Nutrient>,
    foods: &Vec<Food>,
    nutrients_sum: &HashMap<String, NutrientValue>,
) -> Vec<Food> {
    let ideal_nutrients = nutrients
        .iter()
        .map(|n| (
            n.name.clone(), 
            &n.recommended_intake - &nutrients_sum[&n.name],
        ))
        .collect::<HashMap<String, NutrientValue>>();
    let mut sortedFoods = foods.clone();
    sortedFoods
        .sort_by_key(|f| 1);
    sortedFoods
        .into_iter()
        .rev()
        .take(3)
        .collect::<Vec<Food>>()
}

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
        assert_eq!(foods.len(), 2887);
        assert_eq!(foods[0].nutrients.len(), 59);
    }

    #[test]
    fn search_food() -> () {
        let (_nutrients, foods) = get_foods();
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
        let (_nutrients, foods) = get_foods();
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
    fn add_nutrient_values() -> () {
        let nv1 = NutrientValue::Code("N".to_string());
        let nv2 = NutrientValue::Code("N".to_string());
        assert_eq!(
            &nv1 + &nv2,
            NutrientValue::Code("N".to_string())
        );

        let nv3 = NutrientValue::Code("N".to_string());
        let nv4 = NutrientValue::Value(76.5);
        assert_eq!(
            &nv3 + &nv4,
            NutrientValue::Code("N".to_string())
        );

        let nv5 = NutrientValue::Value(76.5);
        let nv6 = NutrientValue::Code("N".to_string());
        assert_eq!(
            &nv5 + &nv6,
            NutrientValue::Code("N".to_string())
        );

        let nv5 = NutrientValue::Value(76.5);
        let nv6 = NutrientValue::Value(20.);
        assert_eq!(
            &nv5 + &nv6,
            NutrientValue::Value(96.5)
        );
    }

    #[test]
    fn sub_nutrient_values() -> () {
        let nv1 = NutrientValue::Code("N".to_string());
        let nv2 = NutrientValue::Code("N".to_string());
        assert_eq!(
            &nv1 - &nv2,
            NutrientValue::Value(0.)
        );

        let nv3 = NutrientValue::Code("N".to_string());
        let nv4 = NutrientValue::Value(76.5);
        assert_eq!(
            &nv3 - &nv4,
            NutrientValue::Code("N".to_string())
        );

        let nv5 = NutrientValue::Value(76.5);
        let nv6 = NutrientValue::Code("N".to_string());
        assert_eq!(
            &nv5 - &nv6,
            NutrientValue::Value(0.)
        );

        let nv5 = NutrientValue::Value(76.5);
        let nv6 = NutrientValue::Value(20.);
        assert_eq!(
            &nv5 - &nv6,
            NutrientValue::Value(56.5)
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
        assert_eq!(
            nutrients_sum["vitamin_c_mg"],
            NutrientValue::Code("N".to_string())
        );
        assert_eq!(
            nutrients_sum["vitamin_b12_ug"],
            NutrientValue::Value(0.)
        );
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
    }
}
