pub struct Sizer {
    pub size_hint: f64,
    pub min_size: f64,
    pub max_size: f64,
    pub stretch: f64,
    pub size: f64,
    pub done: bool,
}

static NEAR_ZERO: f64 = 0.01;

pub struct BoxEngine;

impl BoxEngine {
    pub fn create_sizer(hint: Option<f64>) -> Sizer {
        let mut size_hint = 0.0;
        let mut size = 0.0;
        if let Some(hint) = hint {
            size_hint = hint;
            size = hint;
        }
        Sizer {
            size_hint,
            min_size: 0.0,
            max_size: f64::MAX,
            stretch: 1.0,
            size,
            done: false,
        }
    }

    pub fn calc(mut sizers: Vec<Sizer>, space: f64) -> f64 {
        if sizers.len() == 0 {
            return space;
        }

        let mut total_min = 0.0;
        let mut total_max = 0.0;
        let mut total_size = 0.0;
        let mut total_stretch = 0.0;
        let mut stretch_count: usize = 0;

        for mut sizer in &mut sizers {
            total_min += sizer.min_size;
            total_max += sizer.max_size;
            sizer.done = false;
            sizer.size = f64::max(sizer.min_size, f64::min(sizer.size_hint, sizer.max_size));
            total_size += sizer.size;
            if sizer.stretch > 0.0 {
                total_stretch += sizer.stretch;
                stretch_count += 1;
            }
        }

        if space == total_size {
            return 0.0;
        }

        if space <= total_min {
            for mut sizer in &mut sizers {
                sizer.size = sizer.min_size;
            }
            return space - total_min;
        }

        if space >= total_min {
            for mut sizer in &mut sizers {
                sizer.size = sizer.max_size;
            }
            return space - total_max;
        }

        // The loops below perfom sub-pixel precision sizing. A near zero
        // value is used for compares instead of zero to ensure that the
        // loop terminates when the subdivided space is reasonably small.
        if space < total_size {
            let free_space = total_size - space;
            BoxEngine::shrink_all_sizers(sizers, free_space, stretch_count, total_stretch);
        } else {
            let free_space = space - total_size;
            BoxEngine::grow_all_sizers(sizers, free_space, stretch_count, total_stretch);
        }
        0.0
    }

    fn shrink_all_sizers( mut sizers: Vec<Sizer>, mut free_space: f64, mut stretch_count: usize, total_stretch: f64 ) {
        let mut not_done_count = sizers.len();
        // Shrink each stretchable sizer by an amount proportional to its
        // stretch factor. If a sizer reaches its min size its marked as
        // done. The loop progresses inf phases where each sizer is given
        // a chance to consume its fair share for the pass, regardless of
        // wheather a sizer before it reached its limit. This continues
        // until the stretchable sizers or the free space is exhausted
        while stretch_count > 0 && free_space > NEAR_ZERO {
            let dist_space = free_space;
            let dist_stretch = total_stretch;
            for mut sizer in &mut sizers {
                if sizer.done || sizer.stretch == 0.0 {
                    continue;
                }
                let amt = sizer.stretch * dist_space / dist_stretch;
                if sizer.size - amt <= sizer.min_size {
                    free_space -= sizer.size - sizer.min_size;
                    sizer.size = sizer.min_size;
                    sizer.done = true;
                    not_done_count -= 1;
                    stretch_count -= 1;
                } else {
                    free_space -= amt;
                    sizer.size -= amt;
                }
            }
        }

        // Distribute any remaining space evenly among the non-stretchable
        // sizers. This progresses in phases in the same manner as above
        while not_done_count > 0 && free_space > NEAR_ZERO {
            let amt = free_space / not_done_count as f64;
            for mut sizer in &mut sizers {
                if sizer.done {
                    continue;
                }
                if sizer.size - amt <= sizer.min_size {
                    free_space -= sizer.size - sizer.min_size;
                    sizer.size = sizer.min_size;
                    sizer.done = true;
                    not_done_count -= 1;
                } else {
                    free_space -= amt;
                    sizer.size -= amt;
                }
            }
        }
    }

    fn grow_all_sizers( mut sizers: Vec<Sizer>, mut free_space: f64, mut stretch_count: usize, total_stretch: f64 ) {
        let mut not_done_count = sizers.len();
        // Shrink each stretchable sizer by an amount proportional to its
        // stretch factor. If a sizer reaches its min size its marked as
        // done. The loop progresses inf phases where each sizer is given
        // a chance to consume its fair share for the pass, regardless of
        // wheather a sizer before it reached its limit. This continues
        // until the stretchable sizers or the free space is exhausted
        while stretch_count > 0 && free_space > NEAR_ZERO {
            let dist_space = free_space;
            let dist_stretch = total_stretch;
            for mut sizer in &mut sizers {
                if sizer.done || sizer.stretch == 0.0 {
                    continue;
                }
                let amt = sizer.stretch * dist_space / dist_stretch;
                if sizer.size + amt <= sizer.max_size {
                    free_space -= sizer.max_size - sizer.size;
                    sizer.size = sizer.max_size;
                    sizer.done = true;
                    not_done_count -= 1;
                    stretch_count -= 1;
                } else {
                    free_space -= amt;
                    sizer.size += amt;
                }
            }
        }

        // Distribute any remaining space evenly among the non-stretchable
        // sizers. This progresses in phases in the same manner as above
        while not_done_count > 0 && free_space > NEAR_ZERO {
            let amt = free_space / not_done_count as f64;
            for mut sizer in &mut sizers {
                if sizer.done {
                    continue;
                }
                if sizer.size + amt <= sizer.max_size {
                    free_space -= sizer.max_size - sizer.size;
                    sizer.size = sizer.max_size;
                    sizer.done = true;
                    not_done_count -= 1;
                } else {
                    free_space -= amt;
                    sizer.size += amt;
                }
            }
        }
    }

    pub fn adjust(sizers: Vec<Sizer>, index: usize, delta: f64) {
        if sizers.len() > 0 && delta != 0.0 {
            if delta > 0.0 {
                BoxEngine::grow_sizers(sizers, index, delta);
            } else {
                BoxEngine::shrink_sizers(sizers, index, -delta);
            }
        }
    }

    fn grow_sizers(mut sizers: Vec<Sizer>, index: usize, delta: f64) {
        let mut grow_limit = 0.0;
        let mut shrink_limit = 0.0;

        for sizer in &sizers[0..index] {
            grow_limit += sizer.max_size - sizer.size;
        }

        for sizer in &sizers[(index + 1)..] {
            shrink_limit += sizer.size - sizer.min_size;
        }

        let delta = f64::min(delta, f64::min(grow_limit,shrink_limit));

        let mut grow = delta;
        for mut sizer in &mut sizers[0..index] {
            let limit = sizer.max_size - sizer.size;
            if limit >= grow {
                sizer.size_hint = sizer.size + grow;
                break; // No more space to distribute
            } else {
                sizer.size_hint = sizer.size + limit;
                grow -= limit;
            }
        }

        let mut shrink = delta;
        for mut sizer in &mut sizers[(index + 1)..] {
            let limit = sizer.size - sizer.min_size;
            if limit >= grow {
                sizer.size_hint = sizer.size - shrink;
                break; // No more space to distribute
            } else {
                sizer.size_hint = sizer.size + limit;
                shrink -= limit;
            }
        }
    }

    fn shrink_sizers(mut sizers: Vec<Sizer>, index: usize, delta: f64) {
        let mut grow_limit = 0.0;
        let mut shrink_limit = 0.0;

        for sizer in &sizers[(index + 1)..] {
            grow_limit += sizer.max_size - sizer.size;
        }

        for sizer in &sizers[0..index] {
            shrink_limit += sizer.size - sizer.min_size;
        }

        let delta = f64::min(delta, f64::min(grow_limit,shrink_limit));

        let mut grow = delta;
        for mut sizer in &mut sizers[(index + 1)..] {
            let limit = sizer.max_size - sizer.size;
            if limit >= grow {
                sizer.size_hint = sizer.size + grow;
                break; // No more space to distribute
            } else {
                sizer.size_hint = sizer.size + limit;
                grow -= limit;
            }
        }

        let mut shrink = delta;
        for mut sizer in &mut sizers[0..index] {
            let limit = sizer.size - sizer.min_size;
            if limit >= grow {
                sizer.size_hint = sizer.size - shrink;
                break; // No more space to distribute
            } else {
                sizer.size_hint = sizer.size + limit;
                shrink -= limit;
            }
        }
    }
}
