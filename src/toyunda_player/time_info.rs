#[derive(Debug,Clone,Default,Serialize,Deserialize)]
pub struct TimeInfo {
    #[serde(skip_serializing_if="Option::is_none")]
	/// name of the one who timed
    pub timer: Option<String>,
	#[serde(skip_serializing_if="Option::is_none")]
	/// license : who can reuse this time ?
    pub license: Option<String>,
	#[serde(skip_serializing_if="Option::is_none")]
	/// Is it creditless ?
    pub creditless: Option<bool>,
}
