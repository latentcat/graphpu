use std::rc::Rc;

use csv::Error;
use serde::Deserialize;

use crate::widgets::GraphicDelegation;

#[derive(Debug, Deserialize)]
pub struct Node {
    id: String,
    size: f32,
}

#[derive(Debug, Deserialize)]
pub struct Edge {
    start_id: String,
    end_id: String,
    label: String,
}

pub struct GraphicsModel {
    pub graphic_delegation: Rc<dyn GraphicDelegation>,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

impl GraphicsModel {
    pub fn new(graphic_delegation: Rc<dyn GraphicDelegation>) -> Self {
        Self {
            graphic_delegation,
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }
}

impl GraphicsModel {
    pub fn read_nodes(&mut self, path: &Option<String>) -> Result<(), String> {
        let path = path.as_deref().ok_or("Can't find file")?;
        let err_fomatter = |err| format!("{}", err);

        let mut rdr = csv::Reader::from_path(path).map_err(err_fomatter)?;
        self.nodes.clear();
        for result in rdr.deserialize().map(|result| result.map_err(err_fomatter)) {
            let node: Node = result?;
            self.nodes.push(node);
        }
        println!("{:?}", self.nodes);
        Ok(())
    }

    pub fn read_edges(&mut self, path: &Option<String>) -> Result<(), String>  {
        let path = path.as_deref().ok_or("Can't find file")?;
        let err_fomatter = |err| format!("{}", err);

        let mut rdr = csv::Reader::from_path(path).map_err(err_fomatter)?;
        self.edges.clear();
        for result in rdr.deserialize().map(|result| result.map_err(err_fomatter)) {
            let node: Edge = result?;
            self.edges.push(node);
        }
        println!("{:?}", self.edges);
        Ok(())
    }
}
