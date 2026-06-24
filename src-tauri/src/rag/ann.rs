use instant_distance::{Builder, HnswMap, Point, Search};
use std::sync::Arc;

use super::embeddings::cosine_similarity;

#[derive(Clone)]
struct AnnVector {
    vector: Arc<Vec<f32>>,
}

impl Point for AnnVector {
    fn distance(&self, other: &Self) -> f32 {
        1.0 - cosine_similarity(self.vector.as_slice(), other.vector.as_slice())
    }
}

pub struct RagAnnIndex {
    map: HnswMap<AnnVector, String>,
    len: usize,
}

impl RagAnnIndex {
    pub fn build(records: Vec<(String, Vec<f32>)>) -> Result<Self, String> {
        if records.is_empty() {
            return Err("no embeddings to index".into());
        }

        let dimension = records[0].1.len();
        if dimension == 0 {
            return Err("empty embedding vectors".into());
        }

        let mut points = Vec::with_capacity(records.len());
        let mut values = Vec::with_capacity(records.len());

        for (id, embedding) in records {
            if embedding.len() != dimension {
                return Err("inconsistent embedding dimensions in RAG index".into());
            }
            points.push(AnnVector {
                vector: Arc::new(embedding),
            });
            values.push(id);
        }

        let len = points.len();
        let map = Builder::default().build(points, values);

        Ok(Self { map, len })
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn search(&self, query: &[f32], k: usize) -> Vec<(String, f32)> {
        if k == 0 || query.is_empty() {
            return Vec::new();
        }

        let query_point = AnnVector {
            vector: Arc::new(query.to_vec()),
        };
        let mut search = Search::default();

        self.map
            .search(&query_point, &mut search)
            .take(k)
            .map(|item| {
                let score = 1.0 - item.distance;
                (item.value.clone(), score)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hnsw_finds_nearest_neighbor() {
        let records = vec![
            ("a".into(), vec![1.0, 0.0, 0.0]),
            ("b".into(), vec![0.0, 1.0, 0.0]),
            ("c".into(), vec![0.9, 0.1, 0.0]),
        ];
        let index = RagAnnIndex::build(records).expect("index");
        let hits = index.search(&[1.0, 0.0, 0.0], 2);
        assert!(!hits.is_empty());
        assert_eq!(hits[0].0, "a");
    }
}
