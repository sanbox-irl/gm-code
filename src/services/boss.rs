use std::{collections::BTreeMap, path::PathBuf};

use url::Url;
use yy_boss::{yy_typings::object_yy::EventType, ShaderKind, YyResource};
use yy_boss::{Resource, YypBoss};

use crate::Position;

#[derive(Debug)]
pub struct Boss {
    pub yy_boss: YypBoss,
    pub fpaths_to_lookup_data: BTreeMap<PathBuf, ResourceLookup>,
}

impl Boss {
    pub fn new(folder: &Url) -> Boss {
        let path = folder.to_file_path().unwrap();

        let mut output = None;

        for file in path.read_dir().unwrap() {
            let file = file.unwrap().path();

            if file.extension() == Some(std::ffi::OsStr::new("yyp")) {
                output = Some(file);
            }
        }
        let output = output.unwrap();

        let yy_boss = YypBoss::with_startup_injest(
            output,
            &[Resource::Script, Resource::Object, Resource::Shader],
        )
        .unwrap();

        let mut fpaths_to_lookup_data = BTreeMap::new();

        // parse in every script
        for script in &yy_boss.scripts {
            let output_path = path.join(
                script
                    .yy_resource
                    .relative_yy_directory()
                    .join(script.yy_resource.name())
                    .with_extension("gml"),
            );
            fpaths_to_lookup_data.insert(
                output_path,
                ResourceLookup {
                    name: script.yy_resource.common_data.name.clone(),
                    data: ResourceLookupData::Script,
                },
            );
        }

        // parse in every event object
        for object in &yy_boss.objects {
            let path = path.join(object.yy_resource.relative_yy_directory());
            for event in &object.yy_resource.event_list {
                let (name, number) = event.event_type.filename();

                fpaths_to_lookup_data.insert(
                    path.join(format!("{}_{}.gml", name, number)),
                    ResourceLookup {
                        name: object.yy_resource.name().to_owned(),
                        data: ResourceLookupData::Object(event.event_type),
                    },
                );
            }
        }

        // parse in every shader
        for shader in &yy_boss.shaders {
            for shad_kind in ShaderKind::iter() {
                let output = path.join(
                    shader
                        .yy_resource
                        .relative_yy_directory()
                        .join(shader.yy_resource.name())
                        .with_extension(shad_kind.file_ending()),
                );
                fpaths_to_lookup_data.insert(
                    output,
                    ResourceLookup {
                        name: shader.yy_resource.name().to_owned(),
                        data: ResourceLookupData::Shader(shad_kind),
                    },
                );
            }
        }

        Boss {
            yy_boss,
            fpaths_to_lookup_data,
        }
    }

    pub fn get_text_document<'a>(&self, url: &'a Url) -> Option<&String> {
        self.fpaths_to_lookup_data
            .get(&url.to_file_path().unwrap())
            .and_then(|v| match &v.data {
                ResourceLookupData::Script => self
                    .yy_boss
                    .scripts
                    .get(&v.name)
                    .and_then(|v| v.associated_data.as_ref()),

                ResourceLookupData::Object(event) => self
                    .yy_boss
                    .objects
                    .get(&v.name)
                    .and_then(|v| v.associated_data.as_ref().and_then(|v| v.get(event))),
                ResourceLookupData::Shader(shad_kind) => self
                    .yy_boss
                    .shaders
                    .get(&v.name)
                    .and_then(|v| v.associated_data.as_ref().map(|v| &v[*shad_kind])),
            })
    }

    pub fn get_text_document_mut<'a>(&mut self, url: &'a Url) -> Option<&mut String> {
        unsafe {
            if let Some(v) = self.fpaths_to_lookup_data.get(&url.to_file_path().unwrap()) {
                match &v.data {
                    ResourceLookupData::Script => self
                        .yy_boss
                        .scripts
                        .get_mut(&v.name)
                        .and_then(|v| v.associated_data.as_mut()),

                    ResourceLookupData::Object(event) => self
                        .yy_boss
                        .objects
                        .get_mut(&v.name)
                        .and_then(|v| v.associated_data.as_mut().and_then(|v| v.get_mut(event))),
                    ResourceLookupData::Shader(shad_kind) => self
                        .yy_boss
                        .shaders
                        .get_mut(&v.name)
                        .and_then(|v| v.associated_data.as_mut().map(|v| &mut v[*shad_kind])),
                }
            } else {
                None
            }
        }
    }

    pub fn get_word_in_document<P: Into<Position>>(txt: &str, pos: P) -> Option<&str> {
        let pos: Position = pos.into();

        txt.lines().nth(pos.line).map(|line| {
            // find the last whitespace...
            let mut start = 0;
            for (i, chr) in line.char_indices() {
                if i == pos.column {
                    break;
                }

                if !(chr.is_ascii_alphanumeric() || chr == '_') {
                    start = i + 1;
                }
            }

            // make sure we're not on the last of the line...
            start = pos.column.min(start);

            // FINALLY, the GLORIOUS word is here...
            &line[start..pos.column]
        })
    }

    pub fn get_word_in_document_full<P: Into<Position>>(txt: &str, pos: P) -> Option<&str> {
        let pos: Position = pos.into();

        txt.lines().nth(pos.line).map(|line| {
            // find the last whitespace...
            let mut start = 0;
            let mut end = line.len();

            let mut hit_end = false;
            for (i, chr) in line.char_indices() {
                if !(chr.is_ascii_alphanumeric() || chr == '_') {
                    if hit_end {
                        end = i;
                        break;
                    } else {
                        start = i + 1;
                    }
                }

                if i == pos.column {
                    hit_end = true;
                }
            }

            // make sure we're not on the last of the line...
            start = end.min(start);

            // FINALLY, the GLORIOUS word is here...
            &line[start..end]
        })
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub enum ResourceLookupData {
    Script,
    Object(EventType),
    Shader(ShaderKind),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub struct ResourceLookup {
    pub name: String,
    pub data: ResourceLookupData,
}
