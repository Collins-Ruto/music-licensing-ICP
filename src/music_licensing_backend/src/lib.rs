#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};
use validator::Validate;

// Define type aliases for convenience
type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

// Define the data structures that will be stored in the stable memory
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Song {
    id: u64,
    title: String,
    artist: String,
    owner_id: u64,
    year: u32,
    genre: String,
    price: u32,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Owner {
    id: u64,
    name: String,
    email: String,
    auth_key: String,
    song_ids: Vec<u64>,
    license_ids: Vec<u64>,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct License {
    id: u64,
    song_id: u64,
    owner_id: u64,
    licensee_id: u64,
    approved: bool,
    price: u32,
    start_date: String,
    end_date: String,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Licensee {
    id: u64,
    name: String,
    email: String,
    licenses: Vec<u64>,
}

// Define return types for calls
#[derive(candid::CandidType, Clone, Serialize, Deserialize)]
struct ReturnOwner {
    id: u64,
    name: String,
    email: String,
}

// Implement the 'Storable' trait for each of the data structures
impl Storable for Song {
    // Conversion to bytes
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }
    // Conversion from bytes
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl Storable for Owner {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl Storable for License {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl Storable for Licensee {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

// Implement the 'BoundedStorable' trait for each of the data structures
impl BoundedStorable for Song {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

impl BoundedStorable for Owner {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

impl BoundedStorable for License {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

impl BoundedStorable for Licensee {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

// Define thread-local static variables for memory management and storage
thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static SONG_STORAGE: RefCell<StableBTreeMap<u64, Song, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));

    static OWNER_STORAGE: RefCell<StableBTreeMap<u64, Owner, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
    ));

    static LICENSE_STORAGE: RefCell<StableBTreeMap<u64, License, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
    ));

    static LICENSEE_STORAGE: RefCell<StableBTreeMap<u64, Licensee, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4)))
    ));
}

// Define structs for payload data (used in update calls)
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Validate)]
struct SongPayload {
    #[validate(length(min = 2))]
    title: String,
    artist: String,
    owner_id: u64,
    year: u32,
    genre: String,
    price: u32,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Validate)]
struct OwnerPayload {
    #[validate(length(min = 2))]
    name: String,
    email: String,
    auth_key: String,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize)]
struct LicensePayload {
    song_id: u64,
    licensee_id: u64,
    start_date: String,
    end_date: String,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Validate)]
struct LicenseePayload {
    #[validate(length(min = 2))]
    name: String,
    email: String,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize)]
struct ProtectedPayload {
    auth_key: String,
    license_id: u64,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize)]
struct ApprovePayload {
    auth_key: String,
    license_id: u64,
    cost: u32,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Validate)]
struct UpdateSongPayload {
    auth_key: String,
    id: u64,
    #[validate(length(min = 2))]
    title: String,
    artist: String,
    year: u32,
    genre: String,
    price: u32,
}

// Define query functions to get all licensable songs
#[ic_cdk::query]
fn get_all_songs() -> Result<Vec<Song>, Error> {
    // Retrieve all songs from the storage
    let songs_vec: Vec<(u64, Song)> = SONG_STORAGE.with(|s| s.borrow().iter().collect());
    // Extract the songs from the tuple and create a vector
    let songs: Vec<Song> = songs_vec.into_iter().map(|(_, song)| song).collect();

    // Check if any songs are found
    match songs.len() {
        0 => Err(Error::NotFound {
            msg: format!("no licensable songs could be found"),
        }),
        _ => Ok(songs),
    }
}

// Define query functions to get songs by id
#[ic_cdk::query]
fn get_song(id: u64) -> Result<Song, Error> {
    // Try to get the song by id
    match _get_song(&id) {
        Some(song) => Ok(song),
        None => Err(Error::NotFound {
            msg: format!("song id:{} could not be found", id),
        }),
    }
}

// Helper function to get a song by id
fn _get_song(id: &u64) -> Option<Song> {
    SONG_STORAGE.with(|s| s.borrow().get(id))
}

// Define update functions to create new songs
#[ic_cdk::update]
fn create_song(payload: SongPayload) -> Result<Song, Error> {
    // Validate Payload
    let validate_payload = payload.validate();
    if validate_payload.is_err() {
        return Err(Error::InvalidPayload {
            msg: validate_payload.unwrap_err().to_string(),
        });
    }

    if _get_owner(&payload.owner_id).is_none() {
        return Err(Error::NotFound {
            msg: format!("owner id:{} could not be found, add ownwer first", payload.owner_id.clone()),
        });
    }

    // Increment the global ID counter to get a new unique ID
    let id = ID_COUNTER
        .with(|counter| {
            let current_id = *counter.borrow().get();
            counter.borrow_mut().set(current_id + 1)
        })
        .expect("Cannot increment Ids");

    // Create a new song based on the provided payload
    let song = Song {
        id,
        title: payload.title.clone(),
        artist: payload.artist,
        owner_id: payload.owner_id,
        year: payload.year,
        genre: payload.genre,
        price: payload.price,
    };

    // Check if the owner exists
    match _get_owner(&id) {
        Some(_) => {
            return Err(Error::InvalidPayload {
                msg: format!("owner id:{} could not be found", id),
            })
        }
        None => (),
    }

    // Add the new song to the owner's list of songs
    match add_song_to_owner(song.owner_id, song.id) {
        Ok(_) => (),
        Err(e) => return Err(e),
    }

    // Store the new song in the SONG_STORAGE
    match SONG_STORAGE.with(|s| s.borrow_mut().insert(id, song.clone())) {
        None => Ok(song),
        Some(_) => Err(Error::InvalidPayload {
            msg: format!("song title:{} could not be created", payload.title),
        }),
    }
}

// Define query functions to get owners by id
#[ic_cdk::query]
fn get_song_owner(id: u64) -> Result<ReturnOwner, Error> {
    // Retrieve the song by id
    let song = match _get_song(&id) {
        Some(song) => song,
        None => {
            return Err(Error::NotFound {
                msg: format!("song id:{} could not be found", id),
            })
        }
    };

    // Retrieve the owner of the song
    match _get_owner(&song.owner_id) {
        Some(owner) => Ok(ReturnOwner {
            id: owner.id,
            name: owner.name,
            email: owner.email,
        }),
        None => Err(Error::NotFound {
            msg: format!("owner id:{} could not be found", song.owner_id),
        }),
    }
}

// Define query functions to search songs by title
#[ic_cdk::query]
fn search_song_title_genre_year(title: String) -> Result<Vec<Song>, Error> {
    let query_title = title.to_lowercase();
    // Retrieve all songs from the storage
    let songs_vec: Vec<(u64, Song)> = SONG_STORAGE.with(|s| s.borrow().iter().collect());
    // Extract the songs from the tuple and create a vector
    let songs: Vec<Song> = songs_vec.into_iter().map(|(_, song)| song).collect();
    let mut matching_songs: Vec<Song> = Vec::new();

    // Filter songs for the specified title
    for song in songs {
        if song.title.to_lowercase().contains(&query_title)
            || song.year.to_string().to_lowercase().contains(&query_title)
            || song.genre.to_lowercase().contains(&query_title)
        {
            matching_songs.push(song);
        }
    }

    // Handle cases where no songs are found or return the result
    match matching_songs.len() {
        0 => Err(Error::NotFound {
            msg: format!("no songs could be found with title:{}", title),
        }),
        _ => Ok(matching_songs),
    }
}

// Define update functions to update an existing song
#[ic_cdk::update]
fn update_song(payload: UpdateSongPayload) -> Result<Song, Error> {
    // Validate Payload
    let validate_payload = payload.validate();
    if validate_payload.is_err() {
        return Err(Error::InvalidPayload {
            msg: validate_payload.unwrap_err().to_string(),
        });
    }

    // Retrieve the existing song based on the payload
    let song = match _get_song(&payload.id) {
        Some(song) => song,
        None => {
            return Err(Error::NotFound {
                msg: format!("song id:{} could not be found", payload.id),
            })
        }
    };

    // Retrieve the owner of the song
    let owner = match _get_owner(&song.owner_id) {
        Some(owner) => owner,
        None => {
            return Err(Error::NotFound {
                msg: format!("owner id:{} could not be found", song.owner_id),
            })
        }
    };

    // Check if the provided auth_key matches the owner's auth_key
    if owner.auth_key != payload.auth_key {
        return Err(Error::Unauthorized {
            msg: format!(
                "auth key:{} is invalid, only song owner can update",
                payload.auth_key
            ),
        });
    }

    // Create a new song with updated information
    let mut new_song = song.clone();
    new_song.title = payload.title.clone();
    new_song.artist = payload.artist;
    new_song.year = payload.year;
    new_song.genre = payload.genre;
    new_song.price = payload.price;

    // Store the updated song
    match SONG_STORAGE.with(|s| s.borrow_mut().insert(payload.id, new_song.clone())) {
        Some(_) => Ok(new_song),
        None => Err(Error::InvalidPayload {
            msg: format!(
                "song title:{} id: {} could not be updated",
                payload.title, payload.id
            ),
        }),
    }
}

// Define update functions to delete an existing song
#[ic_cdk::update]
fn delete_song(auth_key: String, id: u64) -> Result<Song, Error> {
    // Retrieve the existing song based on the id
    let song = match _get_song(&id) {
        Some(song) => song,
        None => {
            return Err(Error::NotFound {
                msg: format!("song id:{} could not be found", id),
            })
        }
    };

    // Retrieve the owner of the song
    let owner = match _get_owner(&song.owner_id) {
        Some(owner) => owner,
        None => {
            return Err(Error::NotFound {
                msg: format!("owner id:{} could not be found", song.owner_id),
            })
        }
    };

    // Check if the provided auth_key matches the owner's auth_key
    if owner.auth_key != auth_key {
        return Err(Error::InvalidPayload {
            msg: format!(
                "auth key:{} is invalid, only song owner can delete",
                auth_key
            ),
        });
    }

    // Remove the song from owner's list
    match remove_song_from_owner(id) {
        Ok(_) => (),
        Err(e) => return Err(e),
    }

    // Remove the song from licensee's list
    match remove_song_from_licensee(id) {
        Ok(_) => (),
        Err(e) => return Err(e),
    }

    // Remove the song from the SONG_STORAGE
    match SONG_STORAGE.with(|s| s.borrow_mut().remove(&id)) {
        Some(song) => Ok(song),
        None => Err(Error::InvalidPayload {
            msg: format!("song id:{} could not be deleted", id),
        }),
    }
}

// Helper function to get an owner by id
fn _get_owner(id: &u64) -> Option<Owner> {
    OWNER_STORAGE.with(|s| s.borrow().get(id))
}

// Helper function to add a song to an owner's list
fn add_song_to_owner(owner_id: u64, song_id: u64) -> Result<(), Error> {
    // Retrieve the owner
    let mut owner = match _get_owner(&owner_id) {
        Some(owner) => owner,
        None => {
            return Err(Error::NotFound {
                msg: format!("owner id:{} could not be found", owner_id),
            })
        }
    };

    // Add the song id to the owner's list of song ids
    owner.song_ids.push(song_id);

    // Store the updated owner
    match OWNER_STORAGE.with(|s| s.borrow_mut().insert(owner_id, owner.clone())) {
        Some(_) => Ok(()),
        None => Err(Error::InvalidPayload {
            msg: format!(
                "song id:{} could not be added to owner id:{}",
                song_id, owner_id
            ),
        }),
    }
}

// Define update function to create a new owner
#[ic_cdk::update]
fn create_owner(payload: OwnerPayload) -> Result<Owner, Error> {
    // Validate Payload
    let validate_payload = payload.validate();
    if validate_payload.is_err() {
        return Err(Error::InvalidPayload {
            msg: validate_payload.unwrap_err().to_string(),
        });
    }

    // Increment the global ID counter to get a new unique ID
    let id = ID_COUNTER
        .with(|counter| {
            let current_id = *counter.borrow().get();
            counter.borrow_mut().set(current_id + 1)
        })
        .expect("Cannot increment Ids");

    // Create a new owner instance
    let owner = Owner {
        id,
        name: payload.name.clone(),
        email: payload.email.clone(),
        auth_key: payload.auth_key.clone(),
        song_ids: Vec::new(),
        license_ids: Vec::new(),
    };

    // Insert the owner into the storage and handle potential errors
    match OWNER_STORAGE.with(|s| s.borrow_mut().insert(id, owner.clone())) {
        None => Ok(owner),
        Some(_) => Err(Error::InvalidPayload {
            msg: format!("owner name:{} could not be created", payload.name),
        }),
    }
}

// Define query function to get a license by ID
#[ic_cdk::query]
fn get_license(id: u64) -> Result<License, Error> {
    match _get_license(&id) {
        Some(license) => Ok(license),
        None => Err(Error::NotFound {
            msg: format!("license id:{} could not be found", id),
        }),
    }
}

// Define query function to get license requests for a specific owner
#[ic_cdk::query]
fn get_owner_license_requests(id: u64) -> Result<Vec<License>, Error> {
    // Retrieve all licenses from storage
    let licenses_vec: Vec<(u64, License)> = LICENSE_STORAGE.with(|s| s.borrow().iter().collect());
    let licenses: Vec<License> = licenses_vec
        .into_iter()
        .map(|(_, license)| license)
        .collect();
    let mut owner_licenses: Vec<License> = Vec::new();

    // Filter licenses for the specified owner ID
    for license in licenses {
        if license.owner_id == id {
            owner_licenses.push(license);
        }
    }

    // Handle cases where no licenses are found or return the result
    match owner_licenses.len() {
        0 => Err(Error::NotFound {
            msg: format!("no licenses could be found for owner id:{}", id),
        }),
        _ => Ok(owner_licenses),
    }
}

// Define query function to get licenses for a specific licensee
#[ic_cdk::query]
fn get_licensee_licenses(id: u64) -> Result<Vec<License>, Error> {
    // Retrieve all licenses from storage
    let licenses_vec: Vec<(u64, License)> = LICENSE_STORAGE.with(|s| s.borrow().iter().collect());
    let licenses: Vec<License> = licenses_vec
        .into_iter()
        .map(|(_, license)| license)
        .collect();
    let mut licensee_licenses: Vec<License> = Vec::new();

    // Filter licenses for the specified licensee ID
    for license in licenses {
        if license.licensee_id == id {
            licensee_licenses.push(license);
        }
    }

    // Handle cases where no licenses are found or return the result
    match licensee_licenses.len() {
        0 => Err(Error::NotFound {
            msg: format!("no licenses could be found for licensee id:{}", id),
        }),
        _ => Ok(licensee_licenses),
    }
}

// Define update function to create a new license request
#[ic_cdk::update]
fn create_license_request(payload: LicensePayload) -> Result<License, Error> {
    // Increment the global ID counter to get a new unique ID
    let id = ID_COUNTER
        .with(|counter| {
            let current_id = *counter.borrow().get();
            counter.borrow_mut().set(current_id + 1)
        })
        .expect("Cannot increment Ids");

    // validate licensee
    match _get_licensee(&payload.licensee_id) {
        Some(_) => (),
        None => {
            return Err(Error::NotFound {
                msg: format!("licensee id:{} could not be found, add them first", payload.licensee_id),
            })
        }
    }

    // Retrieve the corresponding song for the license request
    let song = match _get_song(&payload.song_id) {
        Some(song) => song,
        None => {
            return Err(Error::NotFound {
                msg: format!("song id:{} could not be found", payload.song_id),
            })
        }
    };

    // Create a new license instance
    let license = License {
        id,
        song_id: payload.song_id,
        owner_id: song.owner_id,
        licensee_id: payload.licensee_id,
        approved: false,
        price: 0,
        start_date: payload.start_date,
        end_date: payload.end_date,
    };

    // Insert the license request into storage and handle potential errors
    match LICENSE_STORAGE.with(|s| s.borrow_mut().insert(id, license.clone())) {
        None => Ok(license),
        Some(_) => Err(Error::InvalidPayload {
            msg: format!("license id:{} could not be created", id),
        }),
    }
}

// Define update function to approve a license
#[ic_cdk::update]
fn approve_license(payload: ApprovePayload) -> Result<License, Error> {
    // Retrieve the license to be approved
    let license = match _get_license(&payload.license_id) {
        Some(license) => license,
        None => {
            return Err(Error::NotFound {
                msg: format!("license id:{} could not be found", payload.license_id),
            })
        }
    };

    // Retrieve the owner of the license
    let owner = match _get_owner(&license.owner_id) {
        Some(owner) => owner,
        None => {
            return Err(Error::NotFound {
                msg: format!("owner id:{} could not be found", license.owner_id),
            })
        }
    };

    // Validate the authenticity of the approval request
    if owner.auth_key != payload.auth_key {
        return Err(Error::Unauthorized {
            msg: format!(
                "auth key:{} is invalid, only song owner can approve",
                payload.auth_key
            ),
        });
    }

    // Check if the license has already been approved
    if license.approved {
        return Err(Error::AlreadyApproved {
            msg: format!(
                "license id:{} has already been approved",
                payload.license_id
            ),
        });
    }

    // Create a new license with the approval and cost information
    let mut new_license = license.clone();
    new_license.approved = true;
    new_license.price = payload.cost;

    // Update the owner and licensee with the approved license
    match add_license_to_owner(license.owner_id, license.id) {
        Ok(_) => (),
        Err(e) => return Err(e),
    }

    match add_license_to_licensee(license.licensee_id, license.id) {
        Ok(_) => (),
        Err(e) => return Err(e),
    }

    // Update the license in storage and handle potential errors
    match LICENSE_STORAGE.with(|s| {
        s.borrow_mut()
            .insert(payload.license_id, new_license.clone())
    }) {
        Some(_) => Ok(new_license),
        None => Err(Error::InvalidPayload {
            msg: format!("license id:{} could not be approved", payload.license_id),
        }),
    }
}

// Define update function to revoke a license
#[ic_cdk::update]
fn revoke_license(payload: ProtectedPayload) -> Result<License, Error> {
    // Retrieve the license to be revoked
    let license = match _get_license(&payload.license_id) {
        Some(license) => license,
        None => {
            return Err(Error::NotFound {
                msg: format!("license id:{} could not be found", payload.license_id),
            })
        }
    };

    // Retrieve the owner of the license
    let owner = match _get_owner(&license.owner_id) {
        Some(owner) => owner,
        None => {
            return Err(Error::NotFound {
                msg: format!("owner id:{} could not be found", license.owner_id),
            })
        }
    };

    // Validate the authenticity of the revocation request
    if owner.auth_key != payload.auth_key {
        return Err(Error::Unauthorized {
            msg: format!(
                "auth key:{} is invalid, only song owner can revoke",
                payload.auth_key
            ),
        });
    }

    // Create a new license with the approval set to false
    let mut new_license = license.clone();
    new_license.approved = false;

    // Remove the license from owner and licensee
    match remove_license_from_owner(license.owner_id, license.id) {
        Ok(_) => (),
        Err(e) => return Err(e),
    }

    match remove_license_from_licensee(license.licensee_id, license.id) {
        Ok(_) => (),
        Err(e) => return Err(e),
    }

    // Update the license in storage and handle potential errors
    match LICENSE_STORAGE.with(|s| {
        s.borrow_mut()
            .insert(payload.license_id, new_license.clone())
    }) {
        Some(_) => Ok(new_license),
        None => Err(Error::InvalidPayload {
            msg: format!("license id:{} could not be revoked", payload.license_id),
        }),
    }
}

// Helper function to retrieve a license by ID
fn _get_license(id: &u64) -> Option<License> {
    LICENSE_STORAGE.with(|s| s.borrow().get(id))
}

// Define query function to get a licensee by ID
#[ic_cdk::query]
fn get_licensee(id: u64) -> Result<Licensee, Error> {
    match _get_licensee(&id) {
        Some(licensee) => Ok(licensee),
        None => Err(Error::NotFound {
            msg: format!("licensee id:{} could not be found", id),
        }),
    }
}

// Helper function to retrieve a licensee by ID
fn _get_licensee(id: &u64) -> Option<Licensee> {
    LICENSEE_STORAGE.with(|s| s.borrow().get(id))
}

// Define update function to create a new licensee
#[ic_cdk::update]
fn create_licensee(payload: LicenseePayload) -> Result<Licensee, Error> {
    // Validate Payload
    let validate_payload = payload.validate();
    if validate_payload.is_err() {
        return Err(Error::InvalidPayload {
            msg: validate_payload.unwrap_err().to_string(),
        });
    }

    // Increment the global ID counter to get a new unique ID
    let id = ID_COUNTER
        .with(|counter| {
            let current_id = *counter.borrow().get();
            counter.borrow_mut().set(current_id + 1)
        })
        .expect("Cannot increment Ids");

    // Create a new licensee instance
    let licensee = Licensee {
        id,
        name: payload.name.clone(),
        email: payload.email.clone(),
        licenses: Vec::new(),
    };

    // Insert the licensee into storage and handle potential errors
    match LICENSEE_STORAGE.with(|s| s.borrow_mut().insert(id, licensee.clone())) {
        None => Ok(licensee),
        Some(_) => Err(Error::InvalidPayload {
            msg: format!("licensee name:{} could not be created", payload.name),
        }),
    }
}

// Helper function to add a license to an owner
fn add_license_to_owner(owner_id: u64, license_id: u64) -> Result<(), Error> {
    let mut owner = match _get_owner(&owner_id) {
        Some(owner) => owner,
        None => {
            return Err(Error::NotFound {
                msg: format!("owner id:{} could not be found", owner_id),
            })
        }
    };

    owner.license_ids.push(license_id);

    // Insert the updated owner into storage and handle potential errors
    match OWNER_STORAGE.with(|s| s.borrow_mut().insert(owner_id, owner.clone())) {
        Some(_) => Ok(()),
        None => Err(Error::InvalidPayload {
            msg: format!(
                "license id:{} could not be added to owner id:{}",
                license_id, owner_id
            ),
        }),
    }
}

// Helper function to add a license to a licensee
fn add_license_to_licensee(licensee_id: u64, license_id: u64) -> Result<(), Error> {
    let mut licensee = match _get_licensee(&licensee_id) {
        Some(licensee) => licensee,
        None => {
            return Err(Error::NotFound {
                msg: format!("licensee id:{} could not be found", licensee_id),
            })
        }
    };

    licensee.licenses.push(license_id);

    // Insert the updated licensee into storage and handle potential errors
    match LICENSEE_STORAGE.with(|s| s.borrow_mut().insert(licensee_id, licensee.clone())) {
        Some(_) => Ok(()),
        None => Err(Error::InvalidPayload {
            msg: format!(
                "license id:{} could not be added to licensee id:{}",
                license_id, licensee_id
            ),
        }),
    }
}

// Helper function to remove a license from an owner
fn remove_license_from_owner(owner_id: u64, license_id: u64) -> Result<(), Error> {
    let mut owner = match _get_owner(&owner_id) {
        Some(owner) => owner,
        None => {
            return Err(Error::NotFound {
                msg: format!("owner id:{} could not be found", owner_id),
            })
        }
    };

    // Find the index of the license ID in the owner's list
    let mut index = 0;
    let mut found = false;
    for (i, id) in owner.license_ids.iter().enumerate() {
        if *id == license_id {
            index = i;
            found = true;
            break;
        }
    }

    // Handle the case where the license ID is not found in the owner's list
    if !found {
        return Err(Error::NotFound {
            msg: format!(
                "license id:{} could not be found in owner id:{}",
                license_id, owner_id
            ),
        });
    }

    // Remove the license ID from the owner's list
    owner.license_ids.remove(index);

    // Insert the updated owner into storage and handle potential errors
    match OWNER_STORAGE.with(|s| s.borrow_mut().insert(owner_id, owner.clone())) {
        Some(_) => Ok(()),
        None => Err(Error::InvalidPayload {
            msg: format!(
                "license id:{} could not be removed from owner id:{}",
                license_id, owner_id
            ),
        }),
    }
}

// Helper function to remove a license from a licensee
fn remove_license_from_licensee(licensee_id: u64, license_id: u64) -> Result<(), Error> {
    // Retrieve the licensee to update
    let mut licensee = match _get_licensee(&licensee_id) {
        Some(licensee) => licensee,
        None => {
            return Err(Error::NotFound {
                msg: format!("licensee id:{} could not be found", licensee_id),
            })
        }
    };

    // Find the index of the license ID in the licensee's list
    let mut index = 0;
    let mut found = false;
    for (i, id) in licensee.licenses.iter().enumerate() {
        if *id == license_id {
            index = i;
            found = true;
            break;
        }
    }

    // Handle the case where the license ID is not found in the licensee's list
    if !found {
        return Err(Error::NotFound {
            msg: format!(
                "license id:{} could not be found in licensee id:{}",
                license_id, licensee_id
            ),
        });
    }

    // Remove the license ID from the licensee's list
    licensee.licenses.remove(index);

    // Update the licensee in storage and handle potential errors
    match LICENSEE_STORAGE.with(|s| s.borrow_mut().insert(licensee_id, licensee.clone())) {
        Some(_) => Ok(()),
        None => Err(Error::InvalidPayload {
            msg: format!(
                "license id:{} could not be removed from licensee id:{}",
                license_id, licensee_id
            ),
        }),
    }
}

// Helper function to remove a song from an owner
fn remove_song_from_owner(id: u64) -> Result<(), Error> {
    // Retrieve the song to be removed
    let song = _get_song(&id).ok_or(Error::NotFound {
        msg: format!("song id:{} could not be found", id),
    })?;

    // Retrieve the owner of the song
    let mut owner = _get_owner(&song.owner_id).ok_or(Error::NotFound {
        msg: format!("owner id:{} could not be found", song.owner_id),
    })?;

    // Find the index of the song ID in the owner's list
    let index = owner
        .song_ids
        .iter()
        .position(|&x| x == song.id)
        .ok_or(Error::NotFound {
            msg: format!(
                "song id:{} could not be found in owner id:{}",
                song.id, owner.id
            ),
        })?;

    // Remove the song ID from the owner's list
    owner.song_ids.remove(index);

    // Update the owner in storage and handle potential errors
    match OWNER_STORAGE.with(|s| s.borrow_mut().insert(owner.id, owner.clone())) {
        Some(_) => Ok(()),
        None => Err(Error::InvalidPayload {
            msg: format!(
                "song id:{} could not be removed from owner id:{}",
                song.id, owner.id
            ),
        }),
    }
}

// Helper function to remove a song from all associated licensees
fn remove_song_from_licensee(id: u64) -> Result<(), Error> {
    // Retrieve the song to be removed
    let song = _get_song(&id).ok_or(Error::NotFound {
        msg: format!("song id:{} could not be found", id),
    })?;

    // Retrieve all licenses from storage
    let licenses_vec: Vec<(u64, License)> = LICENSE_STORAGE.with(|s| s.borrow().iter().collect());
    let licenses: Vec<License> = licenses_vec
        .into_iter()
        .map(|(_, license)| license)
        .collect();

    // Identify licenses associated with the song
    let mut licenses_to_remove: Vec<License> = Vec::new();
    for license in licenses {
        if license.song_id == song.id {
            licenses_to_remove.push(license);
        }
    }

    // Iterate over associated licenses and remove the song ID from each licensee's list
    for license in licenses_to_remove {
        // Retrieve the licensee to update
        let mut licensee = _get_licensee(&license.licensee_id).ok_or(Error::NotFound {
            msg: format!("licensee id:{} could not be found", license.licensee_id),
        })?;

        // Find the index of the license ID in the licensee's list
        let index = licensee
            .licenses
            .iter()
            .position(|&x| x == license.id)
            .ok_or(Error::NotFound {
                msg: format!(
                    "license id:{} could not be found in licensee id:{}",
                    license.id, licensee.id
                ),
            })?;

        // Remove the license ID from the licensee's list
        licensee.licenses.remove(index);

        // Update the licensee in storage and handle potential errors
        match LICENSEE_STORAGE.with(|s| s.borrow_mut().insert(licensee.id, licensee.clone())) {
            Some(_) => (),
            None => {
                return Err(Error::InvalidPayload {
                    msg: format!(
                        "license id:{} could not be removed from licensee id:{}",
                        license.id, licensee.id
                    ),
                })
            }
        }
    }

    Ok(())
}

// Define an Error enum for handling errors
#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
    AlreadyApproved { msg: String },
    InvalidPayload { msg: String },
    Unauthorized { msg: String },
}

// Candid generator for Candid interface
ic_cdk::export_candid!();
