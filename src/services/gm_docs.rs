use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use url::Url;

/// Creates the gm-manual by parsing in the included JSON.
pub fn create_manual() -> GmManual {
    let docs_txt = include_str!("../../assets/docs.json");

    serde_json::from_str(docs_txt).unwrap()
}

/// The typings for the Entire Manual. This can be read as one massive Json.
#[derive(Debug, Default, Clone, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct GmManual {
    /// The built in functions within the manual created by Yyg.
    pub functions: BTreeMap<String, GmManualFunction>,
    /// The built in variables within the manual created by Yyg.
    pub variables: BTreeMap<String, GmManualVariable>,
    /// Many of the built in constants within the manual created by Yyg. Constants are
    /// difficult to accurately scrape from the documentation, so there will be missing
    /// constants as the scrapper gets better and better at finding them.
    pub constants: BTreeMap<String, GmManualConstant>,
}

/// A function scraped from the Gm Manual.
#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GmManualFunction {
    /// The name of the function
    pub name: String,

    /// The parameters of the function.
    pub parameters: Vec<GmManualFunctionParameter>,

    /// The count of the number of required parameters.
    pub required_parameters: usize,

    /// By `variadic`, we mean if the final parameter can take "infinite" arguments. Examples
    /// are `ds_list_add`, where users can invoke it as `ds_list_add(list, index, 1, 2, 3, 4 /* etc */);`
    pub is_variadic: bool,

    /// The example given in the Manual.
    pub example: String,

    /// The description of what the function does.
    pub description: String,

    /// What the function returns.
    pub returns: String,

    /// The link to the webpage.
    pub link: Url,
}

/// A variable scraped from the GmManual.
#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GmManualVariable {
    /// The name of the variable
    pub name: String,

    /// The example given in the Manual.
    pub example: String,

    /// The description of what the variable does.
    pub description: String,

    /// The type of the variable.
    pub returns: String,

    /// The link to the webpage.
    pub link: Url,
}

/// A parameter and description from the manual. Parameters do not directly indicate if they are optional
/// or variadic -- instead, look at [`GmManualFunction`].
///
/// [`GmManualFunction`]: struct.GmManualFunction.html
#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GmManualFunctionParameter {
    /// The name of the parameter.
    pub parameter: String,

    /// A description given of the parameter.
    pub description: String,
}

/// A constant parsed from the GmManual.
///
/// Because parsing constants is difficult, none of these fields are guarenteed to be non-empty except
/// for [`name`]. Additionally, a constant might have more data than just a description -- if that is the case,
/// additional data will be noted in [`secondary_descriptors`]. As a consequence of this, if the `description`
/// is empty, then `secondary_descriptors` will also always be empty.
#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GmManualConstant {
    /// The name of the constant
    pub name: String,

    /// A description of the constant. This is very rarely an empty string (only "cursor_none").
    pub description: String,

    /// The link to the webpage.
    pub link: Url,

    /// Additional descriptors present. Most of the time, this will be None, but can
    /// have some Descriptors and Values present.
    pub secondary_descriptors: Option<BTreeMap<String, String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_parse() {
        let manual = create_manual();
        let draw_sprite = manual.functions.get("draw_sprite").unwrap();

        assert!(draw_sprite.is_variadic == false);
        assert_eq!(draw_sprite.required_parameters, 4);
    }
}
