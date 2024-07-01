use soroban_sdk::{contracttype, Address, ConversionError, Env, Symbol, TryFromVal, Val, Vec};

#[derive(Clone)]
#[contracttype]
pub struct ApprovalAll {
    pub operator: Address,
    pub owner: Address,
}

#[derive(Clone)]
#[contracttype]
pub enum ApprovalKey {
    All(ApprovalAll),
    ID(i128),
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Balance(Address),
    Nonce(Address),
    Minted(Address),
    Admin,
    Name,
    Symbol,
    URI(i128),
    Approval(ApprovalKey),
    Owner(i128),
    Supply,
}

#[derive(Clone)]
#[contracttype]
pub enum ContentType {
    Image,
    Gif,
    Video,
    MP3,
    Ticket,
    Avatar(AvatarAttributes),
}

#[derive(Clone)]
#[contracttype]
pub struct Trait {
    pub trait_type: Symbol,
    pub value: Symbol,
    pub score: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct AvatarAttributes {
    pub fur: Trait,
    pub clothes: Trait,
    pub eyes: Trait,
    pub mouth: Trait,
    pub background: Trait,
    pub hat: Trait,
    pub earring: Option<Trait>,
}

#[derive(Clone)]
#[contracttype]
pub struct NFT {
    pub id: u64,
    pub uri: Symbol,
    pub creator: Address,
    pub owner: Address,
    pub royalties: u32, // Percentage of transaction
    pub content_type: ContentType,
    pub traits: Vec<Trait>, // Optional traits for avatars or other NFTs
}
