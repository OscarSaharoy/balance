use std::collections::HashMap;
use itertools::Itertools;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;


#[derive(Debug, Clone)]
pub struct Nutrient {
    pub name: String,
    pub display_name: String,
    pub abbreviation: String,
    pub units: String,
    pub recommended_intake: f32,
}


#[derive(Debug, Clone)]
pub struct Food {
    pub name: String,
    pub display_name: String,
    recommend: bool,
    pub emoji: String,
    pub nutrients: HashMap<String, f32>,
}

fn make_food(
    record: csv::StringRecord,
    nutrients: Vec<Nutrient>,
) -> Food {
    let name = record
        .get(0)
        .expect("each row has at least 1 record")
        .to_owned();
    let display_name = record
        .get(1)
        .expect("each row has at least 2 records")
        .to_owned();
    let emoji = record
        .get(2)
        .expect("each row has at least 3 records")
        .to_owned();
    let recommend = match record
        .get(3)
        .expect("each row has at least 4 records") {
        "TRUE" => true,
        _ => false,
    };
    let nutrient_values = std::iter::zip(
        nutrients.iter(),
        record.iter().skip(4)
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
        display_name: display_name,
        recommend: recommend,
        emoji: emoji,
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
            .skip(4)
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
            nutrients.clone(),
        ))
        .collect::<Vec<Food>>();

    (nutrients, foods)
}

pub fn lookup_food(
    foods: &Vec<Food>, search: String
) -> Vec<Food> {
    let matcher = SkimMatcherV2::default();
    let search = search.trim().to_lowercase();
    foods
        .iter()
        .k_largest_by_key(
            5,
            |f| matcher
                .fuzzy_match(&f.display_name, &search)
                .unwrap_or(0) * 100 - f.display_name.len() as i64
        )
        .map(|f| f.clone())
        .collect::<Vec<Food>>()
}

pub fn sum_nutrients(
    nutrients: Vec<Nutrient>, foods: Vec<Food>
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
    nutrients: Vec<Nutrient>,
    foods: &Vec<Food>,
    nutrients_sum: HashMap<String, f32>,
) -> Vec<&Food> {
    foods
        .iter()
        .filter(|f| f.recommend)
        .k_largest_by_key(
            3,
            |f| balance_score(&nutrients, &f, &nutrients_sum)
        )
        .collect::<Vec<&Food>>()
}

pub fn get_highest_and_lowest_nutrients(
    nutrients: Vec<Nutrient>,
    nutrient_values: HashMap<String, f32>
) -> (Nutrient, Nutrient) {
    let rank_nutrient = move |n: &&Nutrient| ( nutrient_values[&n.name] / n.recommended_intake * 1000. ) as usize;
    (
        nutrients
            .iter()
            .filter(|n| n.recommended_intake > 0.1)
            .max_by_key(&rank_nutrient)
            .expect("nutrients is nonempty")
            .clone(),
        nutrients
            .iter()
            .filter(|n| n.recommended_intake > 0.1)
            .min_by_key(&rank_nutrient)
            .expect("nutrients is nonempty")
            .clone(),
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
    fn search_single_food() -> () {
        let (_nutrients, foods) = get_foods();

        assert_eq!(
            super::lookup_food(
                &foods,
                "Ackee".to_string()
            )[0].display_name,
            "Canned Ackee",
        );

        assert_eq!(
            super::lookup_food(
                &foods,
                "Rice Pudding".to_string()
            )[0].display_name,
            "Canned Rice Pudding",
        );

        assert_eq!(
            super::lookup_food(
                &foods,
                "Beef".to_string()
            )[0].display_name,
            "Beef Pie",
        );

        assert_eq!(
            super::lookup_food(
                &foods,
                "baked apple sugar".to_string()
            )[0].display_name,
            "Baked Cooking Apples with Sugar",
        );
    }

    #[test]
    fn sum_nutrients() -> () {
        let (nutrients, foods) = get_foods();
        let found_foods = vec!["Ackee", "Amla", "Apples"]
            .iter()
            .map(|&s|
                super::lookup_food(&foods, s.to_string()).remove(0)
            ).collect::<Vec<super::Food>>();
        let nutrients_sum = super::sum_nutrients(
            nutrients,
            found_foods
        );
        assert_eq!(nutrients_sum["vitamin_c_mg"], 48.);
        assert_eq!(nutrients_sum["vitamin_b12_ug"], 0.);
    }

    #[test]
    fn recommend() -> () {
        let (nutrients, foods) = get_foods();
        let found_foods = Vec::<super::Food>::new();
        let nutrients_sum = super::sum_nutrients(
            nutrients.clone(),
            found_foods
        );
        let mut res = 0.;
        for _ in 0..100 {
            let recommended_foods = super::recommend_foods(
                nutrients.clone(),
                &foods,
                nutrients_sum.clone()
            );
            res += recommended_foods[0].nutrients["vitamin_c_mg"];
        }
        println!("{res}");
    }

    #[test]
    fn highest_and_lowest_nutrients() -> () {
        let (nutrients, foods) = get_foods();
        let ackee = &foods[0];
        assert_eq!(ackee.name, "Ackee, canned, drained");
        let (highest_nutrient, lowest_nutrient) = 
            super::get_highest_and_lowest_nutrients(
                nutrients.clone(), ackee.nutrients.clone()
            );
        assert_eq!(highest_nutrient.name, "vitamin_c_mg");
        assert_eq!(lowest_nutrient.name, "fibre_g");

        let yeast = super::lookup_food(
            &foods,
            "Yeast Extract".to_string()
        ).remove(0);
        let found_foods = vec![yeast; 3];
        let nutrients_sum = super::sum_nutrients(
            nutrients.clone(),
            found_foods
        );
        let (highest_nutrient, lowest_nutrient) = 
            super::get_highest_and_lowest_nutrients(
                nutrients.clone(), nutrients_sum
            );
        assert_eq!(highest_nutrient.name, "folate_ug");
        assert_eq!(lowest_nutrient.name, "fibre_g");
    }
}
