use std::collections::HashMap;
use csv::StringRecord;

#[derive(Debug)]
enum NutrientValue {
    Code(String),
    Value(f32),
}

#[derive(Debug)]
struct Nutrient {
    name: String,
    display_name: String,
    abbreviation: String,
    units: String,
    recommended_intake: f32,
}

#[derive(Debug)]
struct Food {
    name: String,
    nutrients: HashMap<String, NutrientValue>,
}

fn main() -> () {
    let message: String = std::fs::read_to_string(
        "./assets/cofid.csv"
    ).expect("cofid.csv is error free");

    let mut nutrients = Vec::<Nutrient>::new();
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(message.as_bytes());
    let mut headers: Vec<Vec<String>> = rdr
        .records()
        .take(5)
        .map(|r| r
            .expect("cofid.csv is error free")
            .into_iter()
            .map(|s| s.to_owned())
            .collect()
        )
        .collect();

    for i in 0..headers[0].len() {
        let recommended_intake: f32 = headers[4]
            .remove(0)
            .parse()
            .map_or(0., |s| s);
        let n = Nutrient {
            name: headers[0].remove(0),
            display_name: headers[3].remove(0),
            abbreviation: headers[2].remove(0),
            units: headers[1].remove(0),
            recommended_intake: recommended_intake,
        };
        nutrients.push(n);
    }

    return;

    for result in rdr.records() {
        match result {
            Ok(record) => println!("{record:?}"),
            Err(g) => println!("{}", g),
        }
    }
}
