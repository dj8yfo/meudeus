use anyhow::anyhow;
use kdl::KdlNode;
use rgb::RGB8;

#[derive(Debug, Clone, Copy)]
pub struct ColorScheme {
    pub links: Links,
    pub notes: Notes,
}

#[derive(Debug, Clone, Copy)]
pub struct Notes {
    pub tag: RGB8,
    pub special_tag: RGB8,
}
impl TryFrom<&KdlNode> for ColorScheme {
    type Error = anyhow::Error;

    fn try_from(value: &KdlNode) -> Result<Self, Self::Error> {
        let links = value
            .children()
            .ok_or(anyhow!("`elements` should have children"))?
            .get("links")
            .ok_or(anyhow!("no `elements.links` node in config"))?;

        let notes = value
            .children()
            .ok_or(anyhow!("`elements` should have children"))?
            .get("notes")
            .ok_or(anyhow!("no `elements.notes` node in config"))?;

        let links = links.try_into()?;
        let notes = notes.try_into()?;

        Ok(Self { links, notes })
    }
}

impl TryFrom<&KdlNode> for Links {
    type Error = anyhow::Error;

    fn try_from(value: &KdlNode) -> Result<Self, Self::Error> {
        let parent_name = value
            .children()
            .ok_or(anyhow!("`links` should have children"))?
            .get("parent_name")
            .ok_or(anyhow!("no `links.parent_name` node in config"))?;

        let parent_name = parent_name
            .get(0)
            .ok_or(anyhow!("arg not found"))?
            .value()
            .as_string()
            .ok_or(anyhow!("should be string"))?
            .to_string();

        let url = value
            .children()
            .ok_or(anyhow!("`links` should have children"))?
            .get("url")
            .ok_or(anyhow!("no `links.url` node in config"))?;

        let url = url
            .get(0)
            .ok_or(anyhow!("arg not found"))?
            .value()
            .as_string()
            .ok_or(anyhow!("should be string"))?
            .to_string();

        let file = value
            .children()
            .ok_or(anyhow!("`links` should have children"))?
            .get("file")
            .ok_or(anyhow!("no `links.file` node in config"))?;

        let file = file
            .get(0)
            .ok_or(anyhow!("arg not found"))?
            .value()
            .as_string()
            .ok_or(anyhow!("should be string"))?
            .to_string();

        let dir = value
            .children()
            .ok_or(anyhow!("`links` should have children"))?
            .get("dir")
            .ok_or(anyhow!("no `links.dir` node in config"))?;

        let dir = dir
            .get(0)
            .ok_or(anyhow!("arg not found"))?
            .value()
            .as_string()
            .ok_or(anyhow!("should be string"))?
            .to_string();

        let broken = value
            .children()
            .ok_or(anyhow!("`links` should have children"))?
            .get("broken")
            .ok_or(anyhow!("no `links.broken` node in config"))?;

        let broken = broken
            .get(0)
            .ok_or(anyhow!("arg not found"))?
            .value()
            .as_string()
            .ok_or(anyhow!("should be string"))?
            .to_string();

        let code_block = value
            .children()
            .ok_or(anyhow!("`links` should have children"))?
            .get("code_block")
            .ok_or(anyhow!("no `links.code_block` node in config"))?;

        let code_block = code_block
            .get(0)
            .ok_or(anyhow!("arg not found"))?
            .value()
            .as_string()
            .ok_or(anyhow!("should be string"))?
            .to_string();

        let unlisted = value
            .children()
            .ok_or(anyhow!("`links` should have children"))?
            .get("unlisted")
            .ok_or(anyhow!("no `links.unlisted` node in config"))?;

        let unlisted = unlisted
            .get(0)
            .ok_or(anyhow!("arg not found"))?
            .value()
            .as_string()
            .ok_or(anyhow!("should be string"))?
            .to_string();

        let cycle = value
            .children()
            .ok_or(anyhow!("`links` should have children"))?
            .get("cycle")
            .ok_or(anyhow!("no `links.cycle` node in config"))?;

        let cycle = cycle
            .get(0)
            .ok_or(anyhow!("arg not found"))?
            .value()
            .as_string()
            .ok_or(anyhow!("should be string"))?
            .to_string();

        let parent_name = serde_json::from_str(&parent_name)?;
        let url = serde_json::from_str(&url)?;
        let file = serde_json::from_str(&file)?;
        let dir = serde_json::from_str(&dir)?;
        let broken = serde_json::from_str(&broken)?;
        let code_block = serde_json::from_str(&code_block)?;
        let unlisted = serde_json::from_str(&unlisted)?;
        let cycle = serde_json::from_str(&cycle)?;

        Ok(Self {
            parent_name,
            url,
            file,
            dir,
            broken,
            code_block,
            unlisted,
            cycle,
        })
    }
}
#[derive(Debug, Clone, Copy)]
pub struct Links {
    pub parent_name: RGB8,
    pub url: RGB8,
    pub file: RGB8,
    pub dir: RGB8,
    pub broken: RGB8,
    pub code_block: RGB8,
    pub unlisted: RGB8,
    pub cycle: RGB8,
}

impl TryFrom<&KdlNode> for Notes {
    type Error = anyhow::Error;

    fn try_from(value: &KdlNode) -> Result<Self, Self::Error> {
        let tag = value
            .children()
            .ok_or(anyhow!("`notes` should have children"))?
            .get("tag")
            .ok_or(anyhow!("no `notes.tag` node in config"))?;

        let tag = tag
            .get(0)
            .ok_or(anyhow!("arg not found"))?
            .value()
            .as_string()
            .ok_or(anyhow!("should be string"))?
            .to_string();

        let special_tag = value
            .children()
            .ok_or(anyhow!("`notes` should have children"))?
            .get("special_tag")
            .ok_or(anyhow!("no `notes.special_tag` node in config"))?;

        let special_tag = special_tag
            .get(0)
            .ok_or(anyhow!("arg not found"))?
            .value()
            .as_string()
            .ok_or(anyhow!("should be string"))?
            .to_string();

        let tag = serde_json::from_str(&tag)?;
        let special_tag = serde_json::from_str(&special_tag)?;

        Ok(Self { tag, special_tag })
    }
}
