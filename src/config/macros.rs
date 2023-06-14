#[macro_export]
macro_rules! impl_try_from_kdl_node {
    ($type: ident, $parent: expr, $($field: ident),+ ) => (

        impl TryFrom<&KdlNode> for $type {
            type Error = miette::Report;

            fn try_from(value: &KdlNode) -> Result<Self, Self::Error> {
                let result = {
                    let tags = [$(stringify!($field)),+];
                    let mut hashmap: HashMap<&'static str, &'_ KdlNode> = HashMap::new();
                    for tag in tags {

                        let node = value
                            .children()
                            .ok_or(KdlNodeErrorType {
                                        err_span: value.span().clone(),
                                        description: format!("`{}` should have children", $parent),
                                    })
                            .map_err(|err| Into::<miette::Report>::into(err))?
                            .get(tag)
                            .ok_or(KdlNodeErrorType {
                                        err_span: value.span().clone(),
                                        description: format!("no `{}.{}` in config", $parent, tag),
                                    })
                            .map_err(|err| Into::<miette::Report>::into(err))?;
                        hashmap.insert(tag, node);
                    }
                    $type {
                        $($field: hashmap[stringify!($field)].try_into()?),+

                    }
                };

                Ok(result)

            }
        }
    )
}
#[macro_export]
macro_rules! impl_try_from_kdl_node_tagged {
($type: ident, $parent: expr, $($tag: expr => $field: ident),+ ) => (

    impl TryFrom<&KdlNode> for $type {
        type Error = miette::Report;

        fn try_from(value: &KdlNode) -> Result<Self, Self::Error> {
            let result = {
                let tags = [$($tag),+];
                let mut hashmap: HashMap<&'static str, &'_ KdlNode> = HashMap::new();
                for tag in tags {

                    let node = value
                        .children()
                        .ok_or(KdlNodeErrorType {
                                    err_span: value.span().clone(),
                                    description: format!("`{}` should have children", $parent),
                                })
                        .map_err(|err| Into::<miette::Report>::into(err))?
                        .get(tag)
                        .ok_or(KdlNodeErrorType {
                                    err_span: value.span().clone(),
                                    description: format!("no `{}.{}` in config", $parent, tag),
                                })
                        .map_err(|err| Into::<miette::Report>::into(err))?;
                    hashmap.insert(tag, node);
                }
                $type {
                    $($field: hashmap[$tag].try_into()?
                    ),+

                }
            };

            Ok(result)

        }
    }
)
}

pub mod keymap {

    #[macro_export]
    macro_rules! impl_try_from_kdl_node_uniqueness_check {
        ($type: ident, $parent: expr, $($field: ident),+ ) => (

            impl TryFrom<&KdlNode> for $type {
                type Error = miette::Report;

                fn try_from(value: &KdlNode) -> Result<Self, Self::Error> {
                    let result = {
                        let tags = [$(stringify!($field)),+];
                        let mut hashmap: HashMap<&'static str, &'_ KdlNode> = HashMap::new();
                        for tag in tags {

                            let node = value
                                .children()
                                .ok_or(KdlNodeErrorType {
                                            err_span: value.span().clone(),
                                            description: format!("`{}` should have children", $parent),
                                        })
                                .map_err(|err| Into::<miette::Report>::into(err))?
                                .get(tag)
                                .ok_or(KdlNodeErrorType {
                                            err_span: value.span().clone(),
                                            description: format!("no `{}.{}` in config", $parent, tag),
                                        })
                                .map_err(|err| Into::<miette::Report>::into(err))?;
                            hashmap.insert(tag, node);
                        }
                        $type {
                            $($field: hashmap[stringify!($field)].try_into()?),+

                        }
                    };
                    let mut count = 0;
                    let mut keys_hash_set = HashSet::new();
                    {

                    $(
                            count += 1;
                            keys_hash_set.insert(result.$field.clone());

                        )+
                    }
                    if count != keys_hash_set.len() {
                        let err = KdlNodeErrorType {
                            err_span: value.span().clone(),
                            description: "has keys combo repetitions".to_string(),
                        };
                        return Err(Into::<miette::Report>::into(err));
                    }

                    Ok(result)

                }
            }
        )
    }

    #[macro_export]
    macro_rules! impl_from_self_into_action_hashmap {
        ($type: ident, $action_type: ident, $($variant: expr => $field: ident | $skim_action: expr),+ ) => (

            #[derive(Debug, Clone)]
            pub struct Bindings(HashMap<(SingleKey, String), $action_type>);


            impl From<$type> for Bindings {
                fn from(value: $type) -> Self {
                    let mut result = HashMap::new();
                    $(result.insert((value.$field, $skim_action), $variant));+
                    ;

                    Bindings(result)
                }
            }
            impl Bindings {
                pub fn keys_descriptors(&self) -> Vec<String> {

                    self
                        .0
                        .keys()
                        .map(|(key, skim_action)| format!("{}:{}", key.combo, skim_action))
                        .collect::<Vec<_>>()

                }
            }

            impl From<&Bindings> for HashMap<tuikit::key::Key, Action> {
                fn from(value: &Bindings) -> Self {
                    value.0.iter().map(|(k,v)| {
                        (k.0.tui_combo.clone(), v.clone())
                    }).collect::<HashMap<_, _>>()

                }

            }
        )
    }
}
