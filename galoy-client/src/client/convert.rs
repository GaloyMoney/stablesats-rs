use graphql_client::PathFragment;

use super::{queries::*, GaloyTransaction, GaloyTransactions, TxCursor};
use crate::error::*;

impl TryFrom<stablesats_transactions_list::ResponseData> for GaloyTransactions {
    type Error = GaloyClientError;

    fn try_from(response: stablesats_transactions_list::ResponseData) -> Result<Self, Self::Error> {
        let me = response.me.ok_or_else(|| GaloyClientError::GraphQLNested {
            message: "Empty `me` in response data".to_string(),
            path: None,
        })?;

        let transactions =
            me.default_account
                .transactions
                .ok_or_else(|| GaloyClientError::GraphQLNested {
                    message: "Empty `transactions` in response data".to_string(),
                    path: None,
                })?;

        let page_info = transactions.page_info;
        let edges = transactions
            .edges
            .ok_or_else(|| GaloyClientError::GraphQLNested {
                message: "Empty `transaction edges` in response data".to_string(),
                path: None,
            })?;
        let list = edges
            .into_iter()
            .map(|edge| {
                let mut cents_per_unit = edge.node.settlement_price.base;
                cents_per_unit
                    .set_scale(edge.node.settlement_price.offset as u32)
                    .expect("failed to set scale");
                GaloyTransaction {
                    cursor: TxCursor::from(edge.cursor),
                    id: edge.node.id,
                    created_at: edge.node.created_at.0,
                    amount_in_usd_cents: (edge.node.settlement_amount * cents_per_unit).round(),
                    settlement_amount: edge.node.settlement_amount,
                    settlement_currency: edge.node.settlement_currency,
                    settlement_method: edge.node.settlement_via,
                    cents_per_unit,
                    memo: edge.node.memo,
                    direction: edge.node.direction,
                    status: edge.node.status,
                }
            })
            .collect();

        Ok(Self {
            list,
            cursor: page_info.start_cursor.map(TxCursor::from),
            has_more: page_info.has_previous_page,
        })
    }
}

#[derive(Debug)]
pub struct PathString(pub Option<Vec<Option<String>>>);
impl From<Option<Vec<PathFragment>>> for PathString {
    fn from(path_frags: Option<Vec<PathFragment>>) -> Self {
        let mut paths = Vec::new();

        match path_frags {
            None => return Self(Some(paths)),
            Some(frags) => {
                for frag in frags {
                    match frag {
                        PathFragment::Key(key) => {
                            if key.is_empty() {
                                continue;
                            }
                            paths.push(Some(key))
                        }
                        PathFragment::Index(index) => paths.push(Some(index.to_string())),
                    };
                }
            }
        }

        Self(Some(paths))
    }
}
