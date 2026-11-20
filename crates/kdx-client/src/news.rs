//! News requests: 600 to 614.

use anyhow::Result;
use kdx_proto::msg::news::{
    NewsMessage, NewsMessageChange, NewsMessageId, NewsMessages, NewsgroupEntry, NewsgroupId,
    NewsgroupInfo, PostNewsMessage,
};
use kdx_proto::session::MessageType;

use crate::Client;

/// Converts line ends to the CR that news bodies use on the wire: `\r\n`
/// and `\n` both become `\r`.
pub fn to_cr_lines(text: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(text.len());
    let mut i = 0;
    while i < text.len() {
        match text[i] {
            b'\r' if text.get(i + 1) == Some(&b'\n') => {
                out.push(b'\r');
                i += 1;
            }
            b'\n' => out.push(b'\r'),
            byte => out.push(byte),
        }
        i += 1;
    }
    out
}

impl Client {
    /// 605 Get Newsgroup List.
    pub async fn newsgroups(&mut self) -> Result<Vec<NewsgroupEntry>> {
        let reply = self.request(MessageType::GET_NEWSGROUP_LIST, &[]).await?;
        Ok(NewsgroupEntry::decode_list(&reply)?)
    }

    /// 602 Get Newsgroup.
    pub async fn newsgroup(&mut self, group_id: u32) -> Result<NewsgroupInfo> {
        let reply = self
            .request(MessageType::GET_NEWSGROUP, &NewsgroupId(group_id).encode())
            .await?;
        Ok(NewsgroupInfo::decode(&reply)?)
    }

    /// 600 Create Newsgroup.
    ///
    /// Server 1.620 answers with a success Status and lists the group in the
    /// next 606, so this is `None`; a 603 Newsgroup Info reply is `Some`.
    pub async fn create_newsgroup(
        &mut self,
        group: &NewsgroupInfo,
    ) -> Result<Option<NewsgroupInfo>> {
        let reply = self
            .request(MessageType::CREATE_NEWSGROUP, &group.encode()?)
            .await?;
        newsgroup_or_status(&reply)
    }

    /// 604 Set Newsgroup; `None` on a success Status, as from Server 1.620,
    /// `Some` on a 603 Newsgroup Info reply.
    pub async fn set_newsgroup(&mut self, group: &NewsgroupInfo) -> Result<Option<NewsgroupInfo>> {
        let reply = self
            .request(MessageType::SET_NEWSGROUP, &group.encode()?)
            .await?;
        newsgroup_or_status(&reply)
    }

    /// 601 Delete Newsgroup.
    pub async fn delete_newsgroup(&mut self, group_id: u32) -> Result<()> {
        self.request(
            MessageType::DELETE_NEWSGROUP,
            &NewsgroupId(group_id).encode(),
        )
        .await?;
        Ok(())
    }

    /// 607 Get News Messages.
    pub async fn news(&mut self, group_id: u32) -> Result<NewsMessages> {
        let reply = self
            .request(
                MessageType::GET_NEWS_MESSAGES,
                &NewsgroupId(group_id).encode(),
            )
            .await?;
        Ok(NewsMessages::decode(&reply)?)
    }

    /// 609 Get News Message.
    pub async fn news_message(&mut self, group_id: u32, message_id: u32) -> Result<NewsMessage> {
        let request = NewsMessageId {
            group_id,
            message_id,
        }
        .encode();
        let reply = self
            .request(MessageType::GET_NEWS_MESSAGE, &request)
            .await?;
        Ok(NewsMessage::decode(&reply)?.0)
    }

    /// 611 Post News Message; `body` lines may end in `\n`, `\r\n` or `\r`.
    pub async fn post_news(&mut self, group_id: u32, subject: &[u8], body: &[u8]) -> Result<()> {
        self.post_news_id(group_id, subject, body).await?;
        Ok(())
    }

    /// 611 Post News Message, returning the new message's ID.
    ///
    /// The server confirms a post with a 613 News Message Posted to every
    /// user, not a reply. The ID is that of the 613 for `group_id` that
    /// arrived before the server answered the request, or 0 if none did.
    pub async fn post_news_id(
        &mut self,
        group_id: u32,
        subject: &[u8],
        body: &[u8],
    ) -> Result<u32> {
        let request = PostNewsMessage {
            group_id,
            subject: subject.to_vec(),
            body: to_cr_lines(body),
            ..PostNewsMessage::default()
        };
        let txid = self
            .send_txid(MessageType::POST_NEWS_MESSAGE, &request.encode()?)
            .await?;
        self.barrier(MessageType::POST_NEWS_MESSAGE, txid).await?;
        let posted = self.events.iter().position(|(header, payload)| {
            header.kind == MessageType::NEWS_MESSAGE_POSTED
                && NewsMessageChange::decode(payload).is_ok_and(|c| c.group_id == group_id)
        });
        let Some(at) = posted else {
            return Ok(0);
        };
        let (_, payload) = self.events.remove(at).expect("found above");
        Ok(NewsMessageChange::decode(&payload)?.message_id)
    }

    /// 612 Delete News Message.
    pub async fn delete_news(&mut self, group_id: u32, message_id: u32) -> Result<()> {
        let request = NewsMessageId {
            group_id,
            message_id,
        }
        .encode();
        self.send_checked(MessageType::DELETE_NEWS_MESSAGE, &request)
            .await
    }
}

fn newsgroup_or_status(reply: &[u8]) -> Result<Option<NewsgroupInfo>> {
    if reply.is_empty() {
        return Ok(None);
    }
    Ok(Some(NewsgroupInfo::decode(reply)?))
}
