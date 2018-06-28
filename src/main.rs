#[macro_use]
extern crate clap;
extern crate xmlJSON;
extern crate rustc_serialize;
extern crate data_encoding;
extern crate inflate;
extern crate percent_encoding;

use clap::App;
use xmlJSON::XmlDocument;
use rustc_serialize::json::{ToJson, Json};
use std::str::FromStr;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;

use data_encoding::BASE64;

use inflate::inflate_bytes;
use std::str::from_utf8;
use percent_encoding::{percent_decode};


fn write_file(mut file: &File, content: String){
    file.write(content.as_bytes()).unwrap();
}

///TODO: 
/// [x]. Group by Diagram
/// [ ]. Add <img>
fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let name = matches.value_of("name").expect("No name specified.");
    let assets_path = matches.value_of("assets").expect("No assets specified.");
    let out_filename = matches.value_of("output").unwrap_or("./output/markdown.md");

    let mut contents = String::new();
    File::open(format!("{}/{}.xml", assets_path, name))
        .and_then(|mut file|file.read_to_string(&mut contents))
        .unwrap();

    let doc: XmlDocument = XmlDocument::from_str(&contents[..]).unwrap();
    let json = doc.to_json();
    
    let mut output_file = File::create(out_filename).unwrap_or({
        out_filename.rfind('/')
            .and_then(|idx| {
                let path = &out_filename[0..idx];
                Some(path)
            })
            .and_then(|path| {
                fs::create_dir_all(path).expect("Failed to create folder for output file.");
                Some(path)
            })
            .and_then(|_p| {
                Some(File::create(out_filename).expect("Failed to create output file"))
            }).expect("Invalid output path")
    });

    json.find_path(&["mxfile", "diagram"])
        .and_then(|array| array.as_array())
        .map(|array| array.iter())
        .unwrap()
        .map(|diagram| {
            // json format here: `$` attribute for the tab itself, `_` for the diagram defalted xml.
            // println!("\n\n{}", diagram.pretty());            
            diagram.as_object()
                .map(|obj| {
                    let diagram_name = obj.get("$")
                        .and_then(|obj|obj.as_object())
                        .and_then(|d|d.get("name"))
                        .and_then(|name|name.as_string())
                        .unwrap();
                    let title = format!("\n# {}\n", diagram_name);
                    write_file(&output_file, title);
                    // load image if exists
                    File::open(format!("./assets/Odyssey-{}.png", diagram_name))
                        .and_then(|mut file| {
                            let mut contents: Vec<u8> = Vec::new();
                            file.read_to_end(&mut contents).unwrap();
                            Ok(contents)
                        })
                        .map(|_content| {
                            let base64_data = BASE64.encode(&_content);
                            let img = format!("\n<img src=\"data:image/png;base64,{}\" />\n", base64_data);
                            write_file(&output_file, img);
                        }).unwrap_or(());
                    obj
                })
                .and_then(|obj|obj.get("_"))
                .and_then(|obj|obj.as_string()).unwrap()
        })
        .for_each(|base64_str| {
            let code = BASE64.decode(base64_str.as_bytes()).unwrap();
            let inflated = inflate_bytes(&code).unwrap();
            let url_encoded = from_utf8(&inflated).unwrap();
            let data_xml = percent_decode(url_encoded.as_bytes()).decode_utf8().unwrap().into_owned();
            let diagram_json: Json = XmlDocument::from_str(&data_xml).unwrap().to_json();
            
            // println!("\n\n{}", diagram_json.pretty());
            
            let obj_opt = diagram_json.find_path(&["mxGraphModel", "root", "object"]);
            if obj_opt.is_some() {
                obj_opt.unwrap().as_array()
                    .map(|array| array.iter())
                    .unwrap()
                    .map(|json| json.as_object()
                        .and_then(|json|json.get("$"))
                        .and_then(|json|json.as_object())
                        .unwrap())
                    .for_each(|element| {
                        let id = element.get("id").and_then(|id| id.as_string()).unwrap_or("");
                        if id == "0" {
                            // find first object which represent the tab page, if there is `note` attribute write to the markdown.        
                            let note_about_tab = element.get("note").and_then(|note| note.as_string()).unwrap_or("");
                            write_file(&output_file, format!("\n{}\n", note_about_tab));                            
                        } else if element.contains_key("tooltip") {
                            // only show elements with tooltip attribute.
                            //TODO: group by object and merge the markdown
                            let object_name = element.get("label").and_then(|l|l.as_string()).unwrap();
                            let tooltip_markdown = element.get("tooltip").and_then(|t|t.as_string()).unwrap();
                            if object_name.trim().len() > 0 && tooltip_markdown.trim().len() > 0 {
                                let para = format!("\n## {}\n{}\n", object_name.replace("\n", "").replace("\r", ""), tooltip_markdown);
                                write_file(&output_file, para);
                            }
                            //println!("## {}\n{}", object_name, tooltip_markdown);
                        }
                    });
            }
            //println!("type: {:?}", diagram_json.as_object().unwrap().get("mxGraphModel").unwrap());
        });

    output_file.flush().unwrap();
}
