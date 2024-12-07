use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct Food {
    a: String,
    b: String,
    c: String,
}

fn main() -> () {
    let data = "a,b,c
1,2,3
4,5,6";
    let mut rdr = csv::Reader::from_reader(data.as_bytes());
    for result in rdr.deserialize::<Food>() {
        if let Ok(record) = result {
            println!("{record:?}");
        } else {
            println!("err");
        }
    }
}
