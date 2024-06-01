use super::Emote;

pub fn transform_emotes(emotes: &[twitch_irc::message::Emote]) -> Vec<Emote> {
    emotes
        .iter()
        .map(|e| {
            let id = e.id.clone();
            let url = format!(
                "https://static-cdn.jtvnw.net/emoticons/v2/{}/static/light/2.0",
                &id
            );

            Emote {
                id,
                code: e.code.clone(),
                url,
            }
        })
        .collect()
}
