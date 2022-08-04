use std::{path::PathBuf, rc::Rc, collections::HashMap};

use crate::models::data_model::ExternalData;

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
