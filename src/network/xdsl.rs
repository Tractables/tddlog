//! GeNIe's `.xdsl` networks and Ace's `.inst` evidence files, both XML.
//!
//! An `.xdsl` network lists its nodes under `<nodes>`: each `<cpt id="X">`
//! with its `<state id="..."/>`s, its `<parents>` and its
//! `<probabilities>`, ordered as in a Hugin table. A node of another kind
//! (noisy-max, deterministic, equation) is not a table, and the network is
//! refused. An `.inst` file observes `<inst id="X" value="v"/>` under
//! `<instantiation>`.

use super::scan::{fields, probability};
use super::{Network, xml};

pub(super) fn parse(data: &[u8]) -> Result<Network, String> {
    let root = xml::parse(data)?;
    let nodes = root.child("nodes").ok_or("xdsl: no <nodes>")?;
    let mut network = Network::default();
    let id = |e: &xml::Element| {
        e.attribute("id")
            .map(str::to_string)
            .ok_or_else(|| format!("xdsl: a <{}> without an id", e.name))
    };
    for node in &nodes.children {
        let name = id(node)?;
        if node.name != "cpt" {
            return Err(format!("xdsl: {name} is a {} node, not a table", node.name));
        }
        let states = node
            .children
            .iter()
            .filter(|c| c.name == "state")
            .map(id)
            .collect::<Result<Vec<_>, _>>()?;
        network.declare(&name, states)?;
    }
    for (v, node) in nodes.children.iter().enumerate() {
        let parents = match node.child("parents") {
            Some(p) => fields(&p.text)
                .map(|p| network.variable(p))
                .collect::<Result<Vec<_>, _>>()?,
            None => Vec::new(),
        };
        let probs = node
            .child("probabilities")
            .ok_or_else(|| format!("xdsl: {} has no <probabilities>", network.names[v]))?;
        let probs = fields(&probs.text)
            .map(probability)
            .collect::<Result<Vec<_>, _>>()?;
        network.set_product_table(v, parents, probs)?;
    }
    Ok(network)
}

/// The observations of an `.inst` file, a variable and a value index
/// each, in the network's order. A variable or value the network does not
/// have, or a variable observed twice, is an error.
pub(super) fn inst(data: &[u8], network: &Network) -> Result<Vec<(usize, usize)>, String> {
    let root = xml::parse(data)?;
    if root.name != "instantiation" {
        return Err("evidence: not an <instantiation>".into());
    }
    let mut given: Vec<Option<usize>> = vec![None; network.names.len()];
    for e in &root.children {
        let (Some(v), Some(u), "inst") = (e.attribute("id"), e.attribute("value"), e.name.as_str())
        else {
            return Err(format!("evidence: not an observation: <{}>", e.name));
        };
        let v = network.variable(v)?;
        let u = network.value(v, u)?;
        if given[v].replace(u).is_some() {
            return Err(format!("evidence: {} observed twice", network.names[v]));
        }
    }
    Ok(given
        .into_iter()
        .enumerate()
        .filter_map(|(v, u)| Some((v, u?)))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    const XDSL: &[u8] = b"<?xml version=\"1.0\" encoding=\"ISO-8859-1\"?>
<smile version=\"1.0\" id=\"n\">
  <nodes>
    <cpt id=\"a\">
      <state id=\"x\" />
      <state id=\"y\" />
      <probabilities>0.3 0.7</probabilities>
    </cpt>
    <cpt id=\"b\">
      <state id=\"u\" />
      <state id=\"v\" />
      <state id=\"w\" />
      <parents>a</parents>
      <probabilities>0.9 0.1 0 0.2 0.5 0.30000000000000004</probabilities>
    </cpt>
  </nodes>
</smile>
";

    #[test]
    fn a_cpt_reads_as_a_hugin_table_and_evidence_by_name() {
        let network = parse(XDSL).unwrap();
        assert_eq!(network.names, ["a", "b"]);
        let b = network.tables[1].as_ref().unwrap();
        assert_eq!(b.parents, [0]);
        assert_eq!(b.probs, [0.9, 0.1, 0.0, 0.2, 0.5, 0.30000000000000004]);
        let given = inst(
            b"<instantiation><inst id=\"b\" value=\"w\"/><inst id=\"a\" value=\"x\"/></instantiation>",
            &network,
        )
        .unwrap();
        assert_eq!(given, [(0, 0), (1, 2)]);
        assert!(
            inst(
                b"<instantiation><inst id=\"a\" value=\"z\"/></instantiation>",
                &network
            )
            .is_err()
        );
        let noisy =
            b"<smile><nodes><noisymax id=\"c\"><state id=\"x\"/></noisymax></nodes></smile>";
        assert!(parse(noisy).unwrap_err().contains("c is a noisymax node"));
    }
}
