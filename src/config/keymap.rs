use std::collections::HashMap;

use anyhow::anyhow;
use kdl::KdlNode;

use self::{checkmark::CheckmarkKeymap, stack::StackKeymap, surf::SurfKeymap, explore::ExploreKeymap};
pub mod checkmark;
pub mod single_key;
pub mod stack;
pub mod surf;
pub mod explore;

#[derive(Debug, Clone)]
pub struct Keymap {
    pub surf: SurfKeymap,
    pub checkmark: CheckmarkKeymap,
    pub stack: StackKeymap,
    pub explore: ExploreKeymap,
}


#[macro_export]
macro_rules! impl_try_from_kdl_node_uniqueness_check {
    ($type: ident, $parent: expr, $($field: ident),+ ) => (

        impl TryFrom<&KdlNode> for $type {
            type Error = anyhow::Error;

            fn try_from(value: &KdlNode) -> Result<Self, Self::Error> {
                let result = {
                    let tags = [$(stringify!($field)),+];
                    let mut hashmap: HashMap<&'static str, &'_ KdlNode> = HashMap::new();
                    for tag in tags {

                        let node = value
                            .children()
                            .ok_or(anyhow!("`{}` should have children", $parent))?
                            .get(tag)
                            .ok_or(anyhow!(format!("no `{}` in config", tag)))?;
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
                    return Err(anyhow!("has key repetitions: {:#?}", value));
                }

                Ok(result)

            }
        }
    )
}

#[macro_export]
macro_rules! impl_try_from_kdl_node {
    ($type: ident, $parent: expr, $($field: ident),+ ) => (

        impl TryFrom<&KdlNode> for $type {
            type Error = anyhow::Error;

            fn try_from(value: &KdlNode) -> Result<Self, Self::Error> {
                let result = {
                    let tags = [$(stringify!($field)),+];
                    let mut hashmap: HashMap<&'static str, &'_ KdlNode> = HashMap::new();
                    for tag in tags {

                        let node = value
                            .children()
                            .ok_or(anyhow!("`{}` should have children", $parent))?
                            .get(tag)
                            .ok_or(anyhow!(format!("no `{}.{}` in config", $parent, tag)))?;
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
impl_try_from_kdl_node!(Keymap, "keymap", surf, checkmark, stack, explore);

#[macro_export]
macro_rules! impl_from_self_into_action_hashmap {
    ($type: ident, $action_type: ident, $($variant: expr => $field: ident),+ ) => (

        #[derive(Debug, Clone)]
        pub struct Bindings(HashMap<SingleKey, $action_type>);


        impl TryFrom<$type> for Bindings {
            type Error = anyhow::Error;
            fn try_from(value: $type) -> Result<Self, Self::Error> {
                let mut result = HashMap::new();
                $(result.insert(value.$field, $variant));+
                ;

                Ok(Bindings(result))
            }
        }
        impl Bindings {
            pub fn keys_descriptors(&self) -> Vec<String> {

                self
                    .0
                    .keys()
                    .map(|key| format!("{}:accept", key.combo))
                    .collect::<Vec<_>>()

            }
        }

        impl From<&Bindings> for HashMap<tuikit::key::Key, Action> {
            fn from(value: &Bindings) -> Self {
                value.0.iter().map(|(k,v)| {
                    (k.tui_combo.clone(), v.clone())
                }).collect::<HashMap<_, _>>()

            }

        }
    )
}



