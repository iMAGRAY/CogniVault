use anyhow::Result;

/// Trait for vector ANN engines.
pub trait AnnEngine: Send + Sync {
    /// Adds a vector and returns internal id.
    fn add_vector(&mut self, vec: Vec<f32>) -> Result<usize>;
    /// Search k nearest.
    fn search(&self, query: &[f32], k: usize) -> Result<Vec<usize>>;
}

#[cfg(feature = "ann_hnsw")]
mod hnsw_impl {
    use super::*;
    use hnsw::{Hnsw, Params, Searcher};
    use rand::SeedableRng;
    use rand_pcg::Pcg64;

    pub struct HnswAnn {
        index: Hnsw<Vec<f32>, Pcg64, 32, 64>,
        searcher: Searcher<f32>,
    }

    impl HnswAnn {
        pub fn new(dim: usize) -> Self {
            let params = Params::default().dim(dim as u32);
            Self {
                index: Hnsw::new_params(params),
                searcher: Searcher::default(),
            }
        }
    }

    impl AnnEngine for HnswAnn {
        fn add_vector(&mut self, vec: Vec<f32>) -> Result<usize> {
            Ok(self.index.insert(vec, &mut self.searcher))
        }

        fn search(&self, query: &[f32], k: usize) -> Result<Vec<usize>> {
            let mut dest = vec![Default::default(); k];
            let res = self.index.nearest(query, k.max(10), &mut self.searcher, &mut dest);
            Ok(res.iter().map(|n| n.idx).collect())
        }
    }
    pub use HnswAnn as DefaultAnn;
}

#[cfg(all(feature = "ann_scalar", not(feature = "ann_hnsw")))]
mod scalar_impl {
    use super::*;
    /// Simple brute-force scalar fallback.
    pub struct ScalarAnn {
        dim: usize,
        data: Vec<Vec<f32>>,
    }
    impl ScalarAnn {
        pub fn new(dim: usize) -> Self { Self { dim, data: Vec::new() } }
    }
    impl AnnEngine for ScalarAnn {
        fn add_vector(&mut self, vec: Vec<f32>) -> Result<usize> {
            if vec.len() != self.dim { return Err(anyhow::anyhow!("dim mismatch")); }
            self.data.push(vec);
            Ok(self.data.len()-1)
        }
        fn search(&self, query: &[f32], k: usize) -> Result<Vec<usize>> {
            if query.len() != self.dim { return Err(anyhow::anyhow!("dim mismatch")); }
            let mut ids: Vec<(usize, f32)> = self.data.iter().enumerate().map(|(i,v)| {
                let dist = v.iter().zip(query).map(|(a,b)| (a-b)*(a-b)).sum::<f32>();
                (i, dist)
            }).collect();
            ids.sort_by(|a,b| a.1.partial_cmp(&b.1).unwrap());
            Ok(ids.into_iter().take(k).map(|t| t.0).collect())
        }
    }
    pub use ScalarAnn as DefaultAnn;
}

#[cfg(feature = "ann_hnsw")]
pub type AnnDefault = hnsw_impl::HnswAnn;
#[cfg(all(feature = "ann_scalar", not(feature="ann_hnsw")))]
pub type AnnDefault = scalar_impl::ScalarAnn; 