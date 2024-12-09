use std::collections::HashMap;

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
    recommended_intake: f64,
}

#[derive(Debug)]
struct Food {
    name: String,
    nutrients: HashMap<String, NutrientValue>,
}

fn main() -> () {
    let message: String = match std::fs::read_to_string(
        "./assets/cofid.csv"
    ) {
        Ok(s) => s,
        Err(g) => panic!("{}", g),
    };
    let nutrients = Vec::<Nutrient>::new();
    let mut rdr = csv::ReaderBuilder::new().has_headers(false).from_reader(message.as_bytes());
    let mut headers = rdr.records().take(5);
    while let Some(Ok(result)) = headers.next() {
        for r in &result {
            println!("{r}");
        }
    }

    for result in rdr.records() {
        match result {
            Ok(record) => println!("{record:?}"),
            Err(g) => println!("{}", g),
        }
    }
}
