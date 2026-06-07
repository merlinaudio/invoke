use std::{ops::Neg, time::Duration};

use objc2::rc::Retained;
use objc2_foundation::{NSDate, NSDateInterval, NSTimeInterval};

pub trait DurationExt {
	fn seconds(self) -> Duration;
	fn millis(self) -> Duration;
	fn micros(self) -> Duration;
	fn nanos(self) -> Duration;
}

impl DurationExt for i32 {
	fn seconds(self) -> Duration {
		Duration::from_secs(self as u64)
	}
	fn millis(self) -> Duration {
		Duration::from_millis(self as u64)
	}
	fn micros(self) -> Duration {
		Duration::from_micros(self as u64)
	}
	fn nanos(self) -> Duration {
		Duration::from_nanos(self as u64)
	}
}

pub trait NSDateExt {
	fn ago(self) -> Retained<NSDate>;
}

impl NSDateExt for Duration {
	fn ago(self) -> Retained<NSDate> {
		NSDate::dateWithTimeIntervalSinceNow(self.as_secs_f64().neg())
	}
}
