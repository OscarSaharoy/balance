use std::collections::HashMap;

#[derive(Debug)]
enum NutrientValue {
    Code(String),
    Value(f32),
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
    let mut rdr = csv::Reader::from_reader(message.as_bytes());
    for result in rdr.records() {
        match result {
            Ok(record) => println!("{record:?}"),
            Err(g) => println!("{}", g),
        }
    }
}
