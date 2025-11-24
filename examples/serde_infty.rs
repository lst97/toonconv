use serde_json::json;
fn main() {
    let v = json!({"inf": std::f64::INFINITY, "nan": std::f64::NAN});
    println!("VALUE: {:?}", v);
    println!("TO_STRING: {}", v);
    if let serde_json::Value::Object(map) = &v {
        for (k, v) in map {
            println!("{} => {:?}", k, v);
        }
    }
}
