//! Allows for converting

/// A type that allows for submitting new laser frames as input, and requesting an iterator of
/// laser points as an output.
#[derive(Clone, Debug)]
pub struct Streamer<P> {
    frame: Vec<P>,
    last_point: Option<P>,
    blank_last_point: bool,
    next_start: usize,
}

/// A command to submit to the laser - either a blank point or a regular coloured point.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Point<P> {
    /// A single point ready for submission to a DAC.
    Regular(P),
    /// A point whose colour should be black (blank) when sent to the laser DAC.
    ///
    /// These points are inserted when trying to get from the end of the frame back to the
    /// beginning of the existing frame or some new frame.
    Blank(P),
}

/// An iterator infinitely yielding points that describe the frame in a large cycle.
#[derive(Debug)]
pub struct Points<'a, P> {
    last_point: &'a mut Option<P>,
    blank_last_point: &'a mut bool,
    points: &'a [P],
    next_start: &'a mut usize,
}

impl<P> Streamer<P> {
    /// Initialise the `Streamer` with an empty frame.
    pub fn new() -> Self {
        Self::from_frame(vec![])
    }

    /// Initialise the `Streamer` with some frame.
    pub fn from_frame(frame: Vec<P>) -> Self {
        Streamer {
            frame,
            last_point: None,
            blank_last_point: false,
            next_start: 0,
        }
    }

    /// Submit a new frame to start streaming.
    pub fn submit_frame(&mut self, frame: Vec<P>) {
        self.frame = frame;
        self.next_start = 0;
        self.blank_last_point = true;
    }

    /// Produce an iterator yielding points that cycle points of the frame, starting from the point
    /// that follows the last point that was yielded.
    pub fn next_points(&mut self) -> Points<P> {
        let Streamer {
            ref frame,
            ref mut last_point,
            ref mut blank_last_point,
            ref mut next_start,
        } = *self;
        Points {
            last_point,
            blank_last_point,
            next_start,
            points: &frame[..],
        }
    }
}

impl<P> Default for Streamer<P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, P> Iterator for Points<'a, P>
where
    P: Clone,
{
    type Item = Point<P>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Send a blank from the last point that was emitted.
            if *self.blank_last_point {
                *self.blank_last_point = false;
                if let Some(p) = self.last_point.clone() {
                    return Some(Point::Blank(p));
                }
            }

            // If there are no points, return `None`.
            if self.points.is_empty() {
                return None;
            }

            // If the last point was the last of the frame, signal for a blank.
            let ix = *self.next_start;
            let len = self.points.len();
            if ix >= len {
                *self.blank_last_point = true;
                *self.next_start = 0;
                continue;
            }

            // Otherwise, return a regular point from the current index.
            let p = self.points[ix].clone();
            *self.next_start += 1;
            *self.last_point = Some(p.clone());
            return Some(Point::Regular(p));
        }
    }
}
