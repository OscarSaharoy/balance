use std::collections::HashMap;

#[derive(Debug, Clone)]
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

fn take_headers(reader: &mut csv::Reader<&[u8]>) -> Vec<Nutrient> {
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

    let nutrients = take_headers(&mut reader);
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

