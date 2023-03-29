use indexmap::IndexMap;
use itertools::Itertools;
use std::collections::{HashMap, VecDeque};

pub struct PeaksDetector {
    threshold: f64,
    influence: f64,
    window: VecDeque<f64>,
}

impl PeaksDetector {
    pub fn new(lag: usize, threshold: f64, influence: f64) -> PeaksDetector {
        PeaksDetector {
            threshold,
            influence,
            window: VecDeque::with_capacity(lag),
        }
    }

    // pub fn signal(&mut self, value: f64) -> Option<Peak> {
    //     if self.window.len() < self.window.capacity() {
    //         self.window.push_front(value);
    //         None
    //     } else if let (Some((mean, stddev)), Some(&window_last)) =
    //         (self.stats(), self.window.last())
    //     {
    //         self.window.pop_back();
    //         if (value - mean).abs() > (self.threshold * stddev) {
    //             let next_value = (value * self.influence) + ((1.0 - self.influence) * window_last);
    //             self.window.push_front(next_value);
    //             Some(if value > mean { Peak::High } else { Peak::Low })
    //         } else {
    //             self.window.push_front(value);
    //             None
    //         }
    //     } else {
    //         None
    //     }
    // }
}

// Finds first local maximums (`max_maximums`) in the floating point input
// image. The output maximums are sorted in the decreasing order. The function
// returns the actual number of maximums found.
pub fn maximums(peaks: &mut IndexMap<u64, u64>) -> Option<()> {
    // let (masses, intensities): (Vec<_>, Vec<_>) = peaks.iter().copied().unzip();
    let max = *peaks.last()?.0;
    for i in 0..=max {
        peaks.entry(i).or_default();
    }
    let i = 0;
    while i < peaks.len() {
        peaks.entry(i as _).or_default();
    }
    // let (masses, intensities): (Vec<u64>, Vec<u64>) = peaks.iter().unzip();
    // peaks.iter().map(|(&mass, &intensity)| {});
    // let min = *peaks.first()?.0;
    // let max = *peaks.last()?.0;
    // for key in min..=max {
    //     peaks[&key];
    // }

    Some(())

    // peaks.iter().tuple_windows().map(|(left, middle, right)| {
    // });
    // compute the limit for maximum points in the array
    // for window in masses.windows(3) {
    //     if window[0] < window[1] && peak > peaks[index + 1] {}

    //     // if the value is larger than 2 his neighbours than it is a local
    //     // maximum
    //     // let peak = peaks[index];
    //     // if peak > peaks[index - 1] && peak > peaks[index + 1] {}
    //     // chunk.all(|peak| );
    //     // float pt = GetFPixel( img, x, cy );
    //     // if ( pt > GetFPixel( img, x-1, cy )
    //     //     && pt > GetFPixel( img, x+1, cy )
    //     //     && pt > GetFPixel( img, x-1, py )
    //     //     && pt > GetFPixel( img, x, py )
    //     //     && pt > GetFPixel( img, x+1, py )
    //     //     && pt > GetFPixel( img, x-1, ny )
    //     //     && pt > GetFPixel( img, x, ny )
    //     //     && pt > GetFPixel( img, x+1, ny ) )
    //     // {
    //     //     locs[num].x = x;
    //     //     locs[num].y = index;
    //     //     locs[num++].value = pt;
    //     //     if (num == num_loc)
    //     //         throw ImageException("FindLocalMaximums: too many maximums found");
    //     // }
    // }
}

// int FindLocalMaximums( IplImage* img, MaximumPoint* maximums, int max_maximums) {
//     // compute the limit for maximum points in the image
//     const int num_loc = (img->width/3 + 1)*(img->height/3 + 1);
//     scoped_array<MaximumPoint> locs( new MaximumPoint[num_loc] );

//     int num = 0;
//     for( int y = 1; y < img->height-1; ++y )
//     {
//         int cy = img->widthStep*y;
//         int py = cy - img->widthStep, ny = cy + img->widthStep;
//         for( int x = 1; x < img->width-1; ++x )
//         {
//             float pt = GetFPixel( img, x, cy );

//             // if the pixel is larger than all 8 his neighbours
//             // than it is a local maximum
//             if ( pt > GetFPixel( img, x-1, cy )
//                 && pt > GetFPixel( img, x+1, cy )
//                 && pt > GetFPixel( img, x-1, py )
//                 && pt > GetFPixel( img, x, py )
//                 && pt > GetFPixel( img, x+1, py )
//                 && pt > GetFPixel( img, x-1, ny )
//                 && pt > GetFPixel( img, x, ny )
//                 && pt > GetFPixel( img, x+1, ny ) )
//             {
//                 locs[num].x = x;
//                 locs[num].y = y;
//                 locs[num++].value = pt;

//                 if (num == num_loc)
//                     throw ImageException("FindLocalMaximums: too many maximums found");
//             }
//         }
//     }

//     // sort the temporary array and copy it to the output array
//     int max_found = (num > max_maximums) ? max_maximums : num;
//     std::sort( locs.get(), locs.get() + num, MaximumPoint() );
//     std::copy( locs.get(), locs.get() + max_found, maximums );

//     return max_found;
// }

#[cfg(test)]
mod test {
    use super::*;
    use indexmap::indexmap;

    #[test]
    fn a() {
        let mut input = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3];
    }

    #[test]
    fn test() {
        let mut input = indexmap![9u64 => 0u64, 10 => 1, 11 => 2, 12 => 3];
        // let mut key = 0;
        // while key < input.len() {
        //     input.entry(key as _).or_default();
        //     key += 1;
        // }
        let mut output = Vec::new();
        for (&key, &value) in &input {
            while key > output.len() as _ {
                output.push(0);
            }
            output.push(value);
        }
        let output = input.iter().fold(Vec::new(), |mut output, (&key, &value)| {
            while key > output.len() as _ {
                output.push(0);
            }
            output.push(value);
            output
        });
        println!("output: {output:?}");
    }
}
