use std::{collections::HashMap, path::PathBuf};

use rayon::prelude::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};

use crate::models::{data_model::ExternalData, ImportedData};

pub fn pick_csv() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("Text File", &["txt", "csv"])
        .pick_file()
}

pub fn read_headers_from_csv(
    path: &Option<PathBuf>,
) -> Result<(HashMap<String, usize>, Vec<String>), String> {
    let path = path.as_deref().ok_or("Can't find file")?;
    let err_fomatter = |err| format!("{}", err);

    let mut rdr = csv::Reader::from_path(path).map_err(err_fomatter)?;
    let headers_index_str: Vec<_> = rdr
        .headers()
        .map_err(err_fomatter)?
        .into_iter()
        .map(|s| s.to_string())
        .collect();
    let headers_str_index: HashMap<_, _> = headers_index_str
        .iter()
        .enumerate()
        .map(|(index, value)| (value.clone(), index))
        .collect();
    Ok((headers_str_index, headers_index_str))
}

pub fn read_from_csv(path: &Option<PathBuf>) -> Result<ExternalData, String> {
    let path = path.as_deref().ok_or("Can't find file")?;
    let err_fomatter = |err| format!("{}", err);

    let mut rdr = csv::Reader::from_path(path).map_err(err_fomatter)?;
    let headers_str_index: HashMap<_, _> = rdr
        .headers()
        .map_err(err_fomatter)?
        .into_iter()
        .enumerate()
        .map(|(index, s)| (s.to_string(), index))
        .collect();
    let headers_index_str: Vec<_> = headers_str_index
        .keys()
        .map(|key| key.to_string())
        .collect();
    let data: Vec<Vec<_>> = rdr
        .records()
        .into_iter()
        .map(|record| record.unwrap().into_iter().map(str::to_string).collect())
        .collect();
    Ok(ExternalData {
        headers_str_index,
        headers_index_str,
        data,
    })
}

pub fn load_data(
    node_file_path: &str,
    edge_file_path: &str,
    source_index: usize,
    target_index: usize,
) -> Result<ImportedData, String> {
    let node_data =
        read_from_csv(&Some(PathBuf::from(node_file_path))).unwrap_or(ExternalData::default());
    let edge_data = read_from_csv(&Some(PathBuf::from(edge_file_path)))?;
    let err_mapper = |_| String::from("Source and target isn't uint");
    let (source_list, target_list): (Vec<u32>, Vec<u32>) = edge_data
        .data
        .par_iter()
        .map::<_, Result<(u32, u32), String>>(|item| {
            let source = item[source_index].parse::<u32>().map_err(err_mapper)?;
            let target = item[target_index].parse::<u32>().map_err(err_mapper)?;
            Ok((source, target))
        })
        .collect::<Result<_, _>>()?;
    let max_id = (0..source_list.len())
        .into_par_iter()
        .map(|index| std::cmp::max(source_list[index], target_list[index]))
        .max()
        .unwrap();
    Ok(ImportedData {
        node_file_path: node_file_path.to_string(),
        edge_file_path: edge_file_path.to_string(),
        node_data,
        edge_data,
        source_index,
        target_index,
        max_id,
    })
}
