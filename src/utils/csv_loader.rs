use std::{path::PathBuf, rc::Rc, collections::HashMap};

use crate::models::{data_model::ExternalData, ImportedData};

pub fn pick_csv() -> Option<PathBuf> {
  rfd::FileDialog::new()
      .add_filter("Text File", &["txt", "csv"])
      .pick_file()
}

pub fn read_headers_from_csv(path: &Option<PathBuf>) -> Result<Vec<Rc<String>>, String> {
  let path = path.as_deref().ok_or("Can't find file")?;
  let err_fomatter = |err| format!("{}", err);

  let mut rdr = csv::Reader::from_path(path).map_err(err_fomatter)?;
  Ok(rdr.headers()
      .map_err(err_fomatter)?
      .into_iter()
      .map(|s| Rc::new(s.to_string()))
      .collect()
  )
}

pub fn read_from_csv(path: &Option<PathBuf>) -> Result<ExternalData, String> {
  let path = path.as_deref().ok_or("Can't find file")?;
  let err_fomatter = |err| format!("{}", err);

  let mut rdr = csv::Reader::from_path(path).map_err(err_fomatter)?;
  let data_headers: Vec<_> = rdr
      .headers()
      .map_err(err_fomatter)?
      .into_iter()
      .map(|s| Rc::new(s.to_string()))
      .collect();
  let data: Vec<HashMap<_, _>> = rdr
      .records()
      .into_iter()
      .map(|record| {
          data_headers
              .iter()
              .map(|s| s.clone())
              .zip(record.unwrap().into_iter().map(str::to_string))
              .collect()
      })
      .collect();
  Ok(ExternalData { data_headers, data })
}

pub fn load_data(node_file_path: &str, edge_file_path: &str, edge_source: usize, edge_target: usize) -> Result<ImportedData, String> {
  let node_data = read_from_csv(&Some(PathBuf::from(node_file_path))).unwrap_or(ExternalData::default());
  let edge_data = read_from_csv(&Some(PathBuf::from(edge_file_path)))?;
  let source_key = edge_data.data_headers[edge_source].clone();
  let target_key = edge_data.data_headers[edge_target].clone();
  let err_mapper = |_| String::from("Source and target isn't uint");
  let max_id = *edge_data
      .data
      .iter()
      .map::<Result<usize, String>, _>(|item| {
          let source = item
              .get(&source_key)
              .unwrap()
              .parse::<usize>()
              .map_err(err_mapper)?;
          let target = item
              .get(&target_key)
              .unwrap()
              .parse::<usize>()
              .map_err(err_mapper)?;
          Ok(std::cmp::max(source, target))
      })
      .collect::<Result<Vec<_>, _>>()?
      .iter()
      .max()
      .unwrap();
  Ok(ImportedData {
    node_file_path: node_file_path.to_string(),
    edge_file_path: edge_file_path.to_string(),
    node_data,
    edge_data,
    source_key,
    target_key,
    max_id,
})
}