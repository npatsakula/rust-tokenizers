// Copyright 2018 Google AI, Google Brain and Carnegie Mellon University Authors.
// Copyright 2018-2020 The HuggingFace Inc. team.
// Copyright 2019-2020 Guillaume Becquin
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//     http://www.apache.org/licenses/LICENSE-2.0
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::error::TokenizerError;
use crate::vocab::base_vocab::{
    open_protobuf_file, read_special_token_mapping_file, register_as_special_value,
    swap_key_values, SpecialTokenMap,
};
use crate::vocab::Vocab;
use std::collections::HashMap;
use std::path::Path;

/// # XLMRoBERTa Vocab
/// Vocabulary for XLMRoBERTa tokenizer. Contains the following special values:
/// - PAD token
/// - BOS token
/// - EOS token
/// - SEP token
/// - MASK token
/// - CLS token
///
/// Expects a SentencePiece protobuf file when created from file.
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone)]
pub struct XLMRobertaVocab {
    /// A mapping of tokens as string to indices (i.e. the encoder base)
    pub values: HashMap<String, i64>,

    /// A mapping of token IDs to strings (i.e. the decoder base)
    pub indices: HashMap<i64, String>,

    /// Special tokens used by the vocabulary
    pub special_token_map: SpecialTokenMap,

    /// A mapping of special value tokens as strings to IDs (i.e. the encoder base for special
    /// values), special values typically include things like BOS/EOS markers, class markers, mask
    /// markers and padding markers
    pub special_values: HashMap<String, i64>,

    /// A mapping of special value tokens as IDs to strings (i.e. the decoder base for special values)
    pub special_indices: HashMap<i64, String>,
}

const DEFAULT_UNK_TOKEN: &str = "<unk>";
const DEFAULT_PAD_TOKEN: &str = "<pad>";
const DEFAULT_BOS_TOKEN: &str = "<s>";
const DEFAULT_SEP_TOKEN: &str = "</s>";
const DEFAULT_CLS_TOKEN: &str = "<s>";
const DEFAULT_EOS_TOKEN: &str = "</s>";
const DEFAULT_MASK_TOKEN: &str = "<mask>";

impl XLMRobertaVocab {
    pub fn get_pad_value(&self) -> &str {
        self.special_token_map
            .pad_token
            .as_deref()
            .unwrap_or(DEFAULT_PAD_TOKEN)
    }

    pub fn get_bos_value(&self) -> &str {
        self.special_token_map
            .bos_token
            .as_deref()
            .unwrap_or(DEFAULT_BOS_TOKEN)
    }

    pub fn get_sep_value(&self) -> &str {
        self.special_token_map
            .sep_token
            .as_deref()
            .unwrap_or(DEFAULT_SEP_TOKEN)
    }

    pub fn get_cls_value(&self) -> &str {
        self.special_token_map
            .cls_token
            .as_deref()
            .unwrap_or(DEFAULT_CLS_TOKEN)
    }

    pub fn get_eos_value(&self) -> &str {
        self.special_token_map
            .eos_token
            .as_deref()
            .unwrap_or(DEFAULT_EOS_TOKEN)
    }

    pub fn get_mask_value(&self) -> &str {
        self.special_token_map
            .mask_token
            .as_deref()
            .unwrap_or(DEFAULT_MASK_TOKEN)
    }
}

impl Vocab for XLMRobertaVocab {
    fn get_unknown_value(&self) -> &str {
        &self.special_token_map.unk_token
    }

    fn values(&self) -> &HashMap<String, i64> {
        &self.values
    }

    fn indices(&self) -> &HashMap<i64, String> {
        &self.indices
    }

    fn special_values(&self) -> &HashMap<String, i64> {
        &self.special_values
    }

    fn special_indices(&self) -> &HashMap<i64, String> {
        &self.special_indices
    }

    fn from_file<P: AsRef<Path>>(path: P) -> Result<XLMRobertaVocab, TokenizerError> {
        let proto = open_protobuf_file(path)?;

        let special_token_map = SpecialTokenMap {
            unk_token: DEFAULT_UNK_TOKEN.to_string(),
            pad_token: Some(DEFAULT_PAD_TOKEN.to_string()),
            bos_token: Some(DEFAULT_BOS_TOKEN.to_string()),
            sep_token: Some(DEFAULT_SEP_TOKEN.to_string()),
            cls_token: Some(DEFAULT_CLS_TOKEN.to_string()),
            eos_token: Some(DEFAULT_EOS_TOKEN.to_string()),
            mask_token: Some(DEFAULT_MASK_TOKEN.to_string()),
            additional_special_tokens: None,
        };

        let mut values = HashMap::new();
        values.insert(
            special_token_map.cls_token.as_ref().unwrap().clone(),
            values.len() as i64,
        );
        values.insert(
            special_token_map.pad_token.as_ref().unwrap().clone(),
            values.len() as i64,
        );
        values.insert(
            special_token_map.eos_token.as_ref().unwrap().clone(),
            values.len() as i64,
        );
        values.insert(special_token_map.unk_token.clone(), values.len() as i64);
        for piece in proto.get_pieces().iter().skip(3) {
            values.insert(piece.get_piece().to_owned(), values.len() as i64);
        }
        values.insert(
            special_token_map.mask_token.as_ref().unwrap().clone(),
            values.len() as i64,
        );

        let mut special_values = HashMap::new();
        register_as_special_value(&special_token_map.unk_token, &values, &mut special_values)?;
        register_as_special_value(
            special_token_map.bos_token.as_ref().unwrap(),
            &values,
            &mut special_values,
        )?;
        register_as_special_value(
            special_token_map.eos_token.as_ref().unwrap(),
            &values,
            &mut special_values,
        )?;
        register_as_special_value(
            special_token_map.cls_token.as_ref().unwrap(),
            &values,
            &mut special_values,
        )?;
        register_as_special_value(
            special_token_map.mask_token.as_ref().unwrap(),
            &values,
            &mut special_values,
        )?;
        register_as_special_value(
            special_token_map.pad_token.as_ref().unwrap(),
            &values,
            &mut special_values,
        )?;

        let indices = swap_key_values(&values);
        let special_indices = swap_key_values(&special_values);

        Ok(XLMRobertaVocab {
            values,
            indices,
            special_token_map,
            special_values,
            special_indices,
        })
    }

    fn from_file_with_special_token_mapping<P: AsRef<Path>, S: AsRef<Path>>(
        path: P,
        special_token_mapping_path: S,
    ) -> Result<Self, TokenizerError> {
        let proto = open_protobuf_file(path)?;

        let special_token_map = read_special_token_mapping_file(special_token_mapping_path)?;

        let mut values = HashMap::new();
        let mut special_values = HashMap::new();

        if let Some(cls_token) = &special_token_map.cls_token {
            values.insert(cls_token.clone(), values.len() as i64);
            register_as_special_value(cls_token, &values, &mut special_values)?;
        }

        if let Some(pad_token) = &special_token_map.pad_token {
            values.insert(pad_token.clone(), values.len() as i64);
            register_as_special_value(pad_token, &values, &mut special_values)?;
        }

        if let Some(eos_token) = &special_token_map.eos_token {
            values.insert(eos_token.clone(), values.len() as i64);
            register_as_special_value(eos_token, &values, &mut special_values)?;
        }

        values.insert(special_token_map.unk_token.clone(), values.len() as i64);
        for piece in proto.get_pieces().iter().skip(3) {
            values.insert(piece.get_piece().to_owned(), values.len() as i64);
        }
        values.insert(
            special_token_map.cls_token.as_ref().unwrap().clone(),
            values.len() as i64,
        );

        register_as_special_value(&special_token_map.unk_token, &values, &mut special_values)?;
        if let Some(bos_token) = &special_token_map.bos_token {
            register_as_special_value(bos_token, &values, &mut special_values)?;
        }
        if let Some(mask_token) = &special_token_map.mask_token {
            register_as_special_value(mask_token, &values, &mut special_values)?;
        }

        let indices = swap_key_values(&values);
        let special_indices = swap_key_values(&special_values);

        Ok(XLMRobertaVocab {
            values,
            indices,
            special_token_map,
            special_values,
            special_indices,
        })
    }

    fn from_values_and_special_token_map(
        values: HashMap<String, i64>,
        special_token_map: SpecialTokenMap,
    ) -> Result<Self, TokenizerError>
    where
        Self: Sized,
    {
        let mut special_values = HashMap::new();
        special_token_map.register_special_values(&values, &mut special_values)?;

        let indices = swap_key_values(&values);
        let special_indices = swap_key_values(&special_values);
        Ok(Self {
            values,
            indices,
            special_token_map,
            special_values,
            special_indices,
        })
    }

    fn token_to_id(&self, token: &str) -> i64 {
        self._token_to_id(
            token,
            &self.values,
            &self.special_values,
            self.get_unknown_value(),
        )
    }

    fn id_to_token(&self, id: &i64) -> String {
        self._id_to_token(
            id,
            &self.indices,
            &self.special_indices,
            self.get_unknown_value(),
        )
    }
}
