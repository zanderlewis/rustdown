use clap::{App, Arg};
use std::fs::File;
use std::io::{self, Read, Write};
use regex::Regex;

fn main() -> io::Result<()> {
    // Define command-line arguments using clap
    let matches = App::new("rustdown")
        .version("0.0.1")
        .author("Your Name <your.email@example.com>")
        .about("Converts Markdown to HTML")
        .arg(
            Arg::new("input")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .takes_value(true),
        )
        .get_matches();

    // Get the input file path
    let input_file = matches.value_of("input").unwrap();
    let mut file = File::open(input_file)?;
    let mut markdown_input = String::new();
    file.read_to_string(&mut markdown_input)?;

    // Convert Markdown to HTML
    let html_output = markdown_to_html_with_custom_attributes(&markdown_input);

    // Handle output file
    if let Some(output_file) = matches.value_of("output") {
        let mut file = File::create(output_file)?;
        file.write_all(html_output.as_bytes())?;
    } else {
        println!("{}", html_output);
    }

    Ok(())
}

fn markdown_to_html_with_custom_attributes(markdown: &str) -> String {
    let mut html_output = String::new();
    let re = Regex::new(r"\{([^}]+)\}").unwrap();
    let link_re = Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").unwrap();
    let image_re = Regex::new(r"!\[([^\]]*)\]\(([^)]+)\)").unwrap();
    let mut indent_level = 0;

    for line in markdown.lines() {
        let mut html_line = String::new();
        let mut tag = "p"; // Default tag is paragraph
        let mut level = 0;

        if line.starts_with('#') {
            for c in line.chars() {
                if c == '#' {
                    level += 1;
                } else {
                    break;
                }
            }
            if level > 0 && level <= 6 {
                tag = match level {
                    1 => "h1",
                    2 => "h2",
                    3 => "h3",
                    4 => "h4",
                    5 => "h5",
                    6 => "h6",
                    _ => "p", // Fallback to paragraph if level is out of range
                };
            }
        }

        html_line.push_str(&"\t".repeat(indent_level));
        html_line.push('<');
        html_line.push_str(tag);

        if let Some(caps) = re.captures(line) {
            let attributes = &caps[1];
            let mut class = String::new();
            let mut id = String::new();

            for attr in attributes.split_whitespace() {
                if attr.starts_with('.') {
                    class.push_str(&attr[1..]);
                    class.push(' ');
                } else if attr.starts_with('#') {
                    id.push_str(&attr[1..]);
                }
            }

            if !class.is_empty() {
                html_line.push_str(&format!(r#" class="{}""#, class.trim()));
            }

            if !id.is_empty() {
                html_line.push_str(&format!(r#" id="{}""#, id));
            }

            html_line.push('>');
            html_line.push_str(&line[level..line.find('{').unwrap_or(line.len())].trim());
        } else {
            html_line.push('>');
            html_line.push_str(&line[level..].trim());
        }

        // Replace images first, then links
        let mut html_line = image_re.replace_all(&html_line, r#"<img src="$2" alt="$1" />"#).to_string();
        html_line = link_re.replace_all(&html_line, r#"<a href="$2">$1</a>"#).to_string();

        html_line.push_str(&format!("</{}>", tag));
        html_output.push_str(&html_line);
        html_output.push('\n');

        // Adjust indent level for nested tags
        if tag == "div" {
            indent_level += 1;
        } else if tag == "/div" {
            if indent_level > 0 {
                indent_level -= 1;
            }
        }
    }

    html_output
}