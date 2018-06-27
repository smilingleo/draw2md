extern crate xmlJSON;
extern crate rustc_serialize;
extern crate data_encoding;
extern crate inflate;
extern crate percent_encoding;

use xmlJSON::XmlDocument;
use rustc_serialize::json::{ToJson, Json};
use std::str::FromStr;
use std::fs::File;
use std::io::Read;
use std::io::Write;

use data_encoding::BASE64;

use inflate::inflate_bytes;
use std::str::from_utf8;
use percent_encoding::{percent_decode};

fn main() {
    let mut contents = String::new();
    File::open("Odyssey.xml")
        .and_then(|mut file|file.read_to_string(&mut contents))
        .unwrap();

    let doc: XmlDocument = XmlDocument::from_str(&contents[..]).unwrap();
    let json = doc.to_json();
    
    let mut output_file = File::create("/tmp/odyssey.md").unwrap();

    match json.find_path(&["mxfile", "diagram"]) {
        Some(array) => {
            array.as_array().unwrap().iter()
                .map(|item|{
                    item.as_object()
                        .and_then(|obj|obj.get("_"))
                        .and_then(|obj|obj.as_string()).unwrap()
                })
                .for_each(|base64_str| {
                    let code = BASE64.decode(base64_str.as_bytes()).unwrap();
                    let inflated = inflate_bytes(&code).unwrap();
                    let url_encoded = from_utf8(&inflated).unwrap();
                    let data_xml = percent_decode(url_encoded.as_bytes()).decode_utf8().unwrap().into_owned();
                    //println!("{:?}", data_xml);
                    let diagram_json: Json = XmlDocument::from_str(&data_xml).unwrap().to_json();
                    
                    match diagram_json.find_path(&["mxGraphModel", "root", "object"]) {
                        Some(object_array) => {
                            object_array.as_array().unwrap().iter()
                                .map(|json| json.as_object()
                                    .and_then(|json|json.get("$"))
                                    .and_then(|json|json.as_object())
                                    .unwrap())
                                // only show elements with tooltip attribute.
                                .filter(|json| json.contains_key("tooltip"))
                                .for_each(|element| {
                                    //TODO: group by object and merge the markdown
                                    let object_name = element.get("label").and_then(|l|l.as_string()).unwrap();
                                    let tooltip_markdown = element.get("tooltip").and_then(|t|t.as_string()).unwrap();
                                    if object_name.trim().len() > 0 && tooltip_markdown.trim().len() > 0 {
                                        let para = format!("\n## {}\n{}\n", object_name.replace("\n", ""), tooltip_markdown);
                                        output_file.write(para.as_bytes()).unwrap();
                                    }
                                    //println!("## {}\n{}", object_name, tooltip_markdown);
                                });
                        },
                        _ => (),
                    }
                    //println!("type: {:?}", diagram_json.as_object().unwrap().get("mxGraphModel").unwrap());
                });
        },
        None => println!("No diagram found"),
    };
    output_file.flush().unwrap();
}
