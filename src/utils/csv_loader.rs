use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use rayon::prelude::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};

use crate::models::{data_model::ExternalData, ImportedData};

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

pub fn read_from_csv<P: AsRef<Path>>(path: &Option<P>) -> Result<ExternalData, String> {
    let err_fomatter = |err| format!("{}", err);

    let mut rdr =
        csv::Reader::from_path(path.as_ref().ok_or("Can't find file")?).map_err(err_fomatter)?;
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

pub fn load_data<P: AsRef<Path>>(
    node_file_path: P,
    edge_file_path: P,
    source_index: usize,
    target_index: usize,
) -> Result<ImportedData, String> {
    let node_data =
        read_from_csv(&Some(node_file_path.as_ref())).unwrap_or(ExternalData::default());
    let edge_data = read_from_csv(&Some(edge_file_path.as_ref()))?;
    let err_mapper = |_| String::from("Source and target isn't uint");
    let source_target_list = (0..edge_data.data.len() * 2)
        .into_par_iter()
        .map::<_, Result<u32, String>>(|index| {
            let item = &edge_data.data[index / 2];
            if index % 2 == 0 {
                item[source_index].parse::<u32>().map_err(err_mapper)
            } else {
                item[target_index].parse::<u32>().map_err(err_mapper)
            }
        })
        .collect::<Result<Vec<u32>, String>>()?;
    let max_id = *source_target_list.par_iter().max().unwrap();
    Ok(ImportedData {
        node_file_path: node_file_path.as_ref().to_path_buf(),
        edge_file_path: edge_file_path.as_ref().to_path_buf(),
        node_data,
        edge_data,
        source_target_list,
        source_index,
        target_index,
        max_id,
    })
}
