/*
 * Copyright 2007 ZXing authors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

// package com.google.zxing.common;

// import com.google.zxing.NotFoundException;

use crate::{common::Result};
use crate::Exceptions;

use super::{BitMatrix, GridSampler, PerspectiveTransform, Quadrilateral, SamplerControl};

/**
 * @author Sean Owen
 */
#[derive(Default)]
pub struct DefaultGridSampler;

impl GridSampler for DefaultGridSampler {
    fn sample_grid_detailed(
        &self,
        image: &BitMatrix,
        dimensionX: u32,
        dimensionY: u32,
        dst: Quadrilateral,
       src:Quadrilateral
    ) -> Result<BitMatrix> {
        let transform = PerspectiveTransform::quadrilateralToQuadrilateral(
            dst, src,
        );

        self.sample_grid(image, dimensionX, dimensionY, &[SamplerControl::new(dimensionX, dimensionY, transform)])
    }

    fn sample_grid(
        &self,
        image: &BitMatrix,
        dimensionX: u32,
        dimensionY: u32,
        controls: &[SamplerControl]
    ) -> Result<BitMatrix> {
        if dimensionX == 0 || dimensionY == 0 {
            return Err(Exceptions::NOT_FOUND);
        }
        let mut bits = BitMatrix::new(dimensionX, dimensionY)?;
        let mut points = vec![0.0; 2 * dimensionX as usize];
        for y in 0..dimensionY {
            //   for (int y = 0; y < dimensionY; y++) {
            let max = points.len();
            let i_value = y as f32 + 0.5;
            let mut x = 0;
            while x < max {
                // for (int x = 0; x < max; x += 2) {
                points[x] = (x as f32 / 2.0) + 0.5;
                points[x + 1] = i_value;
                x += 2;
            }
            controls.first().unwrap().transform.transform_points_single(&mut points);
            // Quick check to see if points transformed to something inside the image;
            // sufficient to check the endpoints
            self.checkAndNudgePoints(image, &mut points)?;
            // try {
            let mut x = 0;
            while x < max {
                //   for (int x = 0; x < max; x += 2) {
                // if points[x] as u32 >= image.getWidth() || points[x + 1] as u32 >= image.getHeight()
                // {
                //     return Err(Exceptions::notFound(
                //         "index out of bounds, see documentation in file for explanation".to_owned(),
                //     ));
                // }
                if image
                    .try_get(points[x] as u32, points[x + 1] as u32)
                    .ok_or(Exceptions::not_found_with(
                        "index out of bounds, see documentation in file for explanation",
                    ))?
                {
                    // Black(-ish) pixel
                    bits.set(x as u32 / 2, y);
                }
                x += 2;
            }
            // } catch (ArrayIndexOutOfBoundsException aioobe) {
            //   // This feels wrong, but, sometimes if the finder patterns are misidentified, the resulting
            //   // transform gets "twisted" such that it maps a straight line of points to a set of points
            //   // whose endpoints are in bounds, but others are not. There is probably some mathematical
            //   // way to detect this about the transformation that I don't know yet.
            //   // This results in an ugly runtime exception despite our clever checks above -- can't have
            //   // that. We could check each point's coordinates but that feels duplicative. We settle for
            //   // catching and wrapping ArrayIndexOutOfBoundsException.
            //   throw NotFoundException.getNotFoundInstance();
            // }
        }
        Ok(bits)
    }
}
