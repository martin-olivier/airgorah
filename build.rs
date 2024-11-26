use serde::Deserialize;
use std::fs::File;
use std::io::Write;

#[derive(Debug, Deserialize)]
struct RawVendors {
    #[serde(rename = "Mac Prefix")]
    mac_prefix: String,
    #[serde(rename = "Vendor Name")]
    vendor_name: String,
}

fn main() {
    let csv_path = "package/vendors.csv";

    let file_content = std::fs::read_to_string(csv_path).expect("Unable to read CSV file");
    let parsed_data = parse_csv(&file_content);

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = std::path::Path::new(&out_dir).join("vendors.rs");
    let mut file = File::create(dest_path).expect("Unable to create output file");

    write!(
        &mut file,
        "
use lazy_static::lazy_static;

lazy_static! {{
    pub static ref VENDORS: HashMap<&'static str, &'static str> = {{
        let mut map = HashMap::new();
        {}

        map
    }};
}}",
        parsed_data
    )
    .expect("Unable to write to output file");

    println!("cargo:rerun-if-changed={}", csv_path);
}

fn parse_csv(csv_content: &str) -> String {
    let mut code = String::new();
    let mut rdr = csv::ReaderBuilder::new().from_reader(csv_content.as_bytes());
    for result in rdr.deserialize::<RawVendors>().flatten() {
        code.push_str(&format!(
            "map.insert(\"{}\", r#\"{}\"#);\n",
            result.mac_prefix, result.vendor_name
        ));
    }
    code
}
