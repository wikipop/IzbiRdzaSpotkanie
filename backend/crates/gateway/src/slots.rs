use domain::{Day, SlotBitmap, SlotId};
use proto::{ApiError, CreateEventRequest, ErrorCode, HourRange};

/// Turns the slot range of a create-event request into a bitmap.
///
/// Exactly one of the two forms must be used: an explicit `slots` list, or
/// `days` combined with an `hours` range. The result must not be empty.
pub fn resolve(request: &CreateEventRequest) -> Result<SlotBitmap, ApiError> {
    let bitmap = match (&request.slots, &request.days, &request.hours) {
        (Some(slots), None, None) => from_list(slots)?,
        (None, Some(days), Some(hours)) => from_days_and_hours(days, *hours)?,
        _ => {
            return Err(invalid(
                "provide either `slots`, or both `days` and `hours`",
            ));
        }
    };
    if bitmap.is_empty() {
        return Err(invalid("the event must contain at least one slot"));
    }
    Ok(bitmap)
}

fn from_list(slots: &[String]) -> Result<SlotBitmap, ApiError> {
    slots
        .iter()
        .map(|slot| slot.parse::<SlotId>().map_err(|e| invalid(e.to_string())))
        .collect()
}

fn from_days_and_hours(days: &[String], hours: HourRange) -> Result<SlotBitmap, ApiError> {
    if hours.from >= hours.to || hours.to > 24 {
        return Err(invalid(
            "`hours` must satisfy 0 <= from < to <= 24 (`to` is exclusive)",
        ));
    }
    let days = days
        .iter()
        .map(|day| day.parse::<Day>().map_err(|e| invalid(e.to_string())))
        .collect::<Result<Vec<Day>, ApiError>>()?;

    let mut bitmap = SlotBitmap::empty();
    for day in days {
        for hour in hours.from..hours.to {
            bitmap.insert(SlotId::new(day, hour).map_err(|e| invalid(e.to_string()))?);
        }
    }
    Ok(bitmap)
}

fn invalid(message: impl Into<String>) -> ApiError {
    ApiError::new(ErrorCode::InvalidSlot, message)
}
