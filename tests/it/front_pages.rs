//! The front pages' links into the guide: every `guide/#id` the READMEs
//! and the crate's front page link to is an id of the guide, and the
//! front page lists the guide's worked examples in the guide's order.
use std::fs;

const GUIDE: &str = "bindings/wasm/guide/index.html";
const PAGES: [&str; 5] = [
    "README.md",
    "src/lib.rs",
    "bindings/python/README.md",
    "bindings/c/README.md",
    "bindings/wasm/README.md",
];
const LINK: &str = "tddlog/guide/#";

/// What follows each `marker` in `text`, up to the first of `ends`.
fn after<'a>(text: &'a str, marker: &str, ends: &[char]) -> Vec<&'a str> {
    text.match_indices(marker)
        .map(|(i, _)| {
            let rest = &text[i + marker.len()..];
            &rest[..rest.find(ends).unwrap_or(rest.len())]
        })
        .collect()
}

#[test]
fn guide_links_name_its_ids() {
    let guide = fs::read_to_string(GUIDE).unwrap();
    let ids = after(&guide, " id=\"", &['"']);
    for page in PAGES {
        let text = fs::read_to_string(page).unwrap();
        for id in after(&text, LINK, &[')', '"', ' ', '\n']) {
            assert!(
                ids.contains(&id),
                "{page} links to {LINK}{id}, which {GUIDE} does not have"
            );
        }
    }
}

#[test]
fn front_page_lists_the_worked_examples() {
    let guide = fs::read_to_string(GUIDE).unwrap();
    let start = guide
        .find("id=\"examples\"")
        .expect("the guide's worked examples");
    let section = &guide[start..start + guide[start..].find("</section>").unwrap()];
    let examples = after(section, "<h3 id=\"", &['"']);
    // The numbered items of the front page's list, `//! 1. [Title](...#id)`.
    let lib = fs::read_to_string("src/lib.rs").unwrap();
    let listed: Vec<&str> = lib
        .lines()
        .filter(|line| {
            let item = line.trim_start_matches("//! ");
            item.split_once(". [")
                .is_some_and(|(n, _)| n.parse::<usize>().is_ok())
        })
        .flat_map(|line| after(line, LINK, &[')']))
        .collect();
    assert_eq!(
        listed, examples,
        "src/lib.rs's worked examples against {GUIDE}'s"
    );
}
