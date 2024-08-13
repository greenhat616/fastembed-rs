use ndarray::{s, Array, Array2, ArrayView, Dim, Dimension, IxDynImpl};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pooling {
    Cls,
    Mean,
}

impl Default for Pooling {
    /// Change this to define the default pooling strategy.
    ///
    /// Currently this is set to [`Self::Cls`] for backward compatibility.
    fn default() -> Self {
        Self::Cls
    }
}

pub fn cls(tensor: &ArrayView<f32, Dim<IxDynImpl>>) -> anyhow::Result<Array<f32, Dim<IxDynImpl>>> {
    match tensor.dim().ndim() {
        2 => Ok(tensor.to_owned()),
        3 => Ok(tensor.slice(s![.., 0, ..]).to_owned().into_dyn()),
        _ => Err(anyhow::Error::msg(format!(
            "Invalid output shape: {shape:?}. Expected 2D or 3D tensor.",
            shape = tensor.dim()
        ))),
    }
}

/// Pool the previous layer output by taking the element-wise arithmetic mean of the token-level embeddings after applying the attention mask.
/// * `token_embeddings` - token embeddings in form of a tensor output of the encoding.
/// * `attention_mask_array` - is the same mask generated by Tokenizer and used for encoding.
// Please refer to the original python implementation for more details:
// https://github.com/UKPLab/sentence-transformers/blob/c0fc0e8238f7f48a1e92dc90f6f96c86f69f1e02/sentence_transformers/models/Pooling.py#L151
pub fn mean(
    token_embeddings: &ArrayView<f32, Dim<IxDynImpl>>,
    attention_mask_array: Array2<i64>,
) -> anyhow::Result<Array<f32, Dim<IxDynImpl>>> {
    let attention_mask_original_dim = attention_mask_array.dim();

    // Compute attention mask
    let attention_mask = attention_mask_array
        .insert_axis(ndarray::Axis(2))
        .broadcast(token_embeddings.dim())
        .unwrap_or_else(|| {
            panic!(
                "Could not broadcast attention mask from {:?} to {:?}",
                attention_mask_original_dim,
                token_embeddings.dim()
            )
        })
        .mapv(|x| x as f32);

    let masked_tensor = token_embeddings * &attention_mask.view();
    let sum = masked_tensor.sum_axis(ndarray::Axis(1));
    let mask_sum = attention_mask.sum_axis(ndarray::Axis(1));
    let mask_sum = mask_sum.mapv(|x| if x == 0f32 { 1.0 } else { x });
    Ok(&sum / &mask_sum)
}
