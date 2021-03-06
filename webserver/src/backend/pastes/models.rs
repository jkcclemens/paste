use crate::{
  backend::errors::BackendError,
  database::models::{
    deletion_keys::SecretDeletionKey,
    files::File,
    pastes::Paste,
    users::User,
  },
  models::paste::{Content, Visibility},
  utils::Language,
};

use chrono::{DateTime, Utc};

use anyhow::Error;

use std::borrow::Cow;

pub struct PastePayload<'u> {
  pub name: Option<String>,
  pub description: Option<String>,
  pub visibility: Visibility,
  pub expires: Option<DateTime<Utc>>,
  pub author: Option<&'u User>,
  pub files: Vec<FilePayload>,
}

pub struct FilePayload {
  pub name: Option<String>,
  pub highlight_language: Option<Language>,
  pub content: Content,
}

pub struct CreateSuccess {
  pub paste: Paste,
  pub files: Vec<File>,
  pub deletion_key: Option<SecretDeletionKey>,
}

pub enum CreateError {
  NoFiles,
  AnonymousPrivate,
  DuplicateFileNames,
  PasteNameTooLarge,
  PasteNameTooLong,
  PasteDescriptionTooLarge,
  PasteDescriptionTooLong,
  FailedSpamFilter,
  FailedSpamFilterFake(Option<String>),
  FileNameTooLarge,
  FileNameTooLong,
  EmptyFile,
  PastExpirationDate,
  Internal(Error),
}

impl BackendError for CreateError {
  fn into_message(self) -> Result<Cow<'static, str>, Error> {
    let m = match self {
      CreateError::Internal(e) => return Err(e),
      CreateError::NoFiles => "you must upload at least one file",
      CreateError::AnonymousPrivate => "cannot make anonymous private pastes",
      CreateError::DuplicateFileNames => "duplicate file names are not allowed",
      CreateError::PasteNameTooLarge => "paste name must be less than 25 KiB",
      CreateError::PasteNameTooLong => "paste name must be less than or equal to 255 characters",
      CreateError::PasteDescriptionTooLarge => "paste description must be less than 25 KiB",
      CreateError::PasteDescriptionTooLong => "paste description must be less than or equal to 255 characters",
      CreateError::FailedSpamFilter => "the spam filter was triggered",
      CreateError::FailedSpamFilterFake(None) => "an error occurred",
      CreateError::FailedSpamFilterFake(Some(s)) => return Ok(Cow::Owned(s)),
      CreateError::FileNameTooLarge => "file name must be less than 25 KiB",
      CreateError::FileNameTooLong => "file name must be less than or equal to 255 characters",
      CreateError::EmptyFile => "file content must not be empty",
      CreateError::PastExpirationDate => "paste expiry date cannot be in the past",
    };

    Ok(Cow::Borrowed(m))
  }
}
