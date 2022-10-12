// use shared::{payload::*, time::*};

// use super::*;

// impl TryFrom<OkexOrderBook> for OkexBtcUsdSwapOrderBookPayload {
//     type Error = OrderBookError;

//     fn try_from(value: OkexOrderBook) -> Result<Self, Self::Error> {
//         let order_book_data = value
//             .data
//             .first()
//             .ok_or_else(|| OrderBookError::EmptyOrderBook)?;
//         let order_book_action = value.action;

//         Ok(Self(OrderBookPayload {
//             asks: OrderBookRaw::from_exchange_data(order_book_data.asks),
//             bids: OrderBookRaw::from_exchange_data(order_book_data.bids),
//             timestamp: TimeStamp::try_from(order_book_data.ts)?,
//             checksum: CheckSumRaw::try_from(order_book_data.checksum),
//         }))
//     }
// }
