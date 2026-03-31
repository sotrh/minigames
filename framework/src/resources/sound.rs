use std::{io::Cursor, path::Path, sync::Arc};

use rodio::{mixer::Mixer, Decoder, Player};

use crate::resources::load_binary;

#[derive(Debug, Clone, Copy)]
pub struct PlayerId(usize);

#[derive(Debug, Clone, Copy)]
pub struct TrackId(usize);

pub struct Track(Arc<Vec<u8>>);

impl Clone for Track {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl AsRef<[u8]> for Track {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Debug, Default)]
pub struct SoundOptions {
    pub pitch_shift: f32,
}

pub struct SoundSystem {
    handle: rodio::MixerDeviceSink,
    players: Vec<Player>,
    tracks: Vec<Track>,
    finished_players: Vec<PlayerId>,
}

impl SoundSystem {
    pub fn new() -> anyhow::Result<Self> {
        let handle = rodio::DeviceSinkBuilder::open_default_sink()?;
        Ok(Self {
            handle,
            players: Vec::new(),
            tracks: Vec::new(),
            finished_players: Vec::new(),
        })
    }

    pub fn create_player(&mut self) -> PlayerId {
        let id = PlayerId(self.players.len());
        self.players.push(Player::connect_new(&self.handle.mixer()));
        id
    }

    pub async fn load(&mut self, path: impl AsRef<Path>) -> anyhow::Result<TrackId> {
        let path = path.as_ref();

        let id = TrackId(self.tracks.len());

        println!("{}", path.display());
        let bytes = load_binary(path).await?;

        self.tracks.push(Track(Arc::new(bytes)));

        Ok(id)
    }

    pub fn run(&mut self) {
        self.finished_players.clear();
        self.finished_players.extend(
            self.players
                .iter()
                .enumerate()
                .filter_map(|(i, p)| p.empty().then(|| PlayerId(i))),
        );
    }

    pub fn play(&mut self, track_id: TrackId, options: SoundOptions) -> Option<PlayerId> {
        let track = self.tracks.get(track_id.0)?;
        let decoder = Decoder::new(Cursor::new(track.clone())).unwrap();

        

        let player_id = if let Some(id) = self.finished_players.pop() {
            id
        } else {
            self.create_player()
        };

        let player = &self.players[player_id.0];

        player.set_speed(1.0 + options.pitch_shift);
        player.append(decoder);

        Some(player_id)
    }
}
