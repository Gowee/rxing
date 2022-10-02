/*
 * Copyright 2008 ZXing authors
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

use std::{
    collections::HashMap,
    fs::{read_dir, read_to_string, File},
    path::{Path, PathBuf}, io::Write,
};

use rxing::{
    common::HybridBinarizer, BarcodeFormat, BinaryBitmap, BufferedImageLuminanceSource,
    DecodeHintType, DecodeHintValue, RXingResultMetadataType, RXingResultMetadataValue, Reader,
};

use super::TestRXingResult;

use image;

/**
 * @author Sean Owen
 * @author dswitkin@google.com (Daniel Switkin)
 */
pub struct AbstractBlackBoxTestCase {
    test_base: Box<Path>,
    barcode_reader: Box<dyn Reader>,
    expected_format: BarcodeFormat,
    test_rxing_results: Vec<TestRXingResult>,
    hints: HashMap<DecodeHintType, DecodeHintValue>,
}

impl AbstractBlackBoxTestCase {
    pub fn buildTestBase(testBasePathSuffix: &str) -> Box<Path> {
        // A little workaround to prevent aggravation in my IDE
        let test_base = Path::new(testBasePathSuffix);
        // if !testBase.exists() {
        //   // try starting with 'core' since the test base is often given as the project root
        //   testBase = Paths.get("core").resolve(testBasePathSuffix);
        // }
        test_base.to_owned().into()
    }

    pub fn new(
        test_base_path_suffix: &str,
        barcode_reader: Box<dyn Reader>,
        expected_format: BarcodeFormat,
    ) -> Self {
        Self {
            test_base: Self::buildTestBase(test_base_path_suffix),
            barcode_reader,
            expected_format,
            test_rxing_results: Vec::new(),
            hints: HashMap::new(),
        }
    }

    pub fn getTestBase(&self) -> &Box<Path> {
        &self.test_base
    }

    pub fn addTest(&mut self, must_pass_count: u32, try_harder_count: u32, rotation: f32) {
        self.addTestComplex(must_pass_count, try_harder_count, 0, 0, rotation);
    }

    pub fn addHint(&mut self, hint: DecodeHintType, value: DecodeHintValue) {
        self.hints.insert(hint, value);
    }

    /**
     * Adds a new test for the current directory of images.
     *
     * @param mustPassCount The number of images which must decode for the test to pass.
     * @param tryHarderCount The number of images which must pass using the try harder flag.
     * @param maxMisreads Maximum number of images which can fail due to successfully reading the wrong contents
     * @param maxTryHarderMisreads Maximum number of images which can fail due to successfully
     *                             reading the wrong contents using the try harder flag
     * @param rotation The rotation in degrees clockwise to use for this test.
     */
    pub fn addTestComplex(
        &mut self,
        must_pass_count: u32,
        try_harder_count: u32,
        max_misreads: u32,
        max_try_harder_misreads: u32,
        rotation: f32,
    ) {
        self.test_rxing_results.push(TestRXingResult::new(
            must_pass_count,
            try_harder_count,
            max_misreads,
            max_try_harder_misreads,
            rotation,
        ));
    }

    pub fn getImageFiles(&self) -> Vec<PathBuf> {
        assert!(
            self.test_base.exists(),
            "Please download and install test images, and run from the 'core' directory"
        );
        // let paths = Vec::new();
        let path_search = read_dir(&self.test_base);
        const possible_extensions: &str = "jpg,jpeg,gif,png,JPG,JPEG,GIF,PNG";

        let paths = path_search
            .unwrap()
            .into_iter()
            .filter(|r| r.is_ok()) // Get rid of Err variants for Result<DirEntry>
            .map(|r| r.unwrap().path()) // This is safe, since we only have the Ok variants
            .filter(|r| r.is_file()) // Filter out non-folders
            .filter(|r| possible_extensions.contains(r.extension().unwrap().to_str().unwrap()))
            // .map(|r| r.into_boxed_path())
            .collect();

        paths
    }

    pub fn getReader(&self) -> &Box<dyn Reader> {
        &self.barcode_reader
    }

    pub fn testBlackBox(&self) {
        assert!(!self.test_rxing_results.is_empty());

        let image_files = self.getImageFiles();
        let test_count = self.test_rxing_results.len();

        let mut passed_counts = vec![0usize; test_count];
        let mut misread_counts = vec![0usize; test_count];
        let mut try_harder_counts = vec![0usize; test_count];
        let mut try_harder_misread_counts = vec![0usize; test_count];

        for testImage in &image_files {
            // for (Path testImage : imageFiles) {
            log::info(format!("Starting {}", testImage.to_string_lossy()));

            let image = image::open(testImage).unwrap(); //ImageIO.read(testImage.toFile());

            //let testImageFileName = testImage.getFileName().toString();
            let file_base_name = testImage.file_stem().unwrap();
            //let expectedTextFile = self.testBase.resolve(fileBaseName + ".txt");
            let mut expected_text_file = PathBuf::from(testImage.clone());
            expected_text_file.set_extension("txt");
            let expected_text = if expected_text_file.exists() {
                Self::read_file_as_string(expected_text_file)
            } else {
                let mut new_path = self.test_base.clone().to_path_buf();
                new_path.push(file_base_name);
                new_path.set_extension("bin");
                //expectedTextFile = testBase.resolve(fileBaseName + ".bin");
                assert!(new_path.exists());
                Self::read_file_as_string(new_path)
            }
            .unwrap();

            let mut expected_metadata_file: PathBuf = self.test_base.clone().to_path_buf();
            expected_metadata_file.push(format!("{}.metadata", file_base_name.to_str().unwrap()));
            expected_metadata_file.set_extension("txt");
            let expectedMetadata_unfinished = if expected_metadata_file.exists() {
                java_properties::read(
                    std::fs::File::open(expected_metadata_file)
                        .expect("file exists, we already know that"),
                )
                .expect("valid java properties file")
                // try (BufferedReader reader = Files.newBufferedReader(expectedMetadataFile, StandardCharsets.UTF_8)) {
                //   expectedMetadata.load(reader);
                // }
            } else {
                HashMap::new()
            };
            let expected_metadata = HashMap::new();
            for (k, v) in expectedMetadata_unfinished {
                let new_k = RXingResultMetadataType::from(k);
                let new_v = match new_k {
                    RXingResultMetadataType::OTHER => RXingResultMetadataValue::OTHER(v),
                    RXingResultMetadataType::ORIENTATION => {
                        RXingResultMetadataValue::Orientation(v.parse().unwrap_or_default())
                    }
                    RXingResultMetadataType::BYTE_SEGMENTS => {
                        RXingResultMetadataValue::ByteSegments(v.into_bytes())
                    }
                    RXingResultMetadataType::ERROR_CORRECTION_LEVEL => {
                        RXingResultMetadataValue::ErrorCorrectionLevel(v)
                    }
                    RXingResultMetadataType::ISSUE_NUMBER => {
                        RXingResultMetadataValue::IssueNumber(v.parse().unwrap_or_default())
                    }
                    RXingResultMetadataType::SUGGESTED_PRICE => {
                        RXingResultMetadataValue::SuggestedPrice(v)
                    }
                    RXingResultMetadataType::POSSIBLE_COUNTRY => {
                        RXingResultMetadataValue::PossibleCountry(v)
                    }
                    RXingResultMetadataType::UPC_EAN_EXTENSION => {
                        RXingResultMetadataValue::UpcEanExtension(v)
                    }
                    RXingResultMetadataType::PDF417_EXTRA_METADATA => {
                        RXingResultMetadataValue::Pdf417ExtraMetadata(v)
                    }
                    RXingResultMetadataType::STRUCTURED_APPEND_SEQUENCE => {
                        RXingResultMetadataValue::StructuredAppendSequence(
                            v.parse().unwrap_or_default(),
                        )
                    }
                    RXingResultMetadataType::STRUCTURED_APPEND_PARITY => {
                        RXingResultMetadataValue::StructuredAppendParity(
                            v.parse().unwrap_or_default(),
                        )
                    }
                    RXingResultMetadataType::SYMBOLOGY_IDENTIFIER => {
                        RXingResultMetadataValue::SymbologyIdentifier(v)
                    }
                };
            }

            for x in 0..test_count {
                // for (int x = 0; x < testCount; x++) {
                let rotation = self.test_rxing_results.get(x).unwrap().getRotation();
                let rotated_image = Self::rotate_image(&image, rotation);
                let source = BufferedImageLuminanceSource::new(rotated_image);
                let bitmap = BinaryBitmap::new(Box::new(HybridBinarizer::new(Box::new(source))));
                
                // if file_base_name == "09" {
                //     let mut f = File::create("test_file_output.txt").unwrap();
                //     dbg!("dumb");
                //     write!(f,"{}", bitmap.getBlackMatrix().unwrap());
                //     drop(f);
                //     Self::rotate_image(&image, rotation).save("test_image.png").unwrap();
                // }
                
                if let Ok(decoded) =
                    self.decode(&bitmap, rotation, &expected_text, &expected_metadata, false)
                {
                    if decoded {
                        passed_counts[x] += 1;
                    } else {
                        misread_counts[x] += 1;
                    }
                } else {
                    log::fine(format!("could not read at rotation {}", rotation));
                }
                // try {
                //   if (decode(bitmap, rotation, expectedText, expectedMetadata, false)) {
                //     passedCounts[x]+=1;
                //   } else {
                //     misreadCounts[x]+=1;
                //   }
                // } catch (ReaderException ignored) {
                //   log::fine(format!("could not read at rotation {}", rotation));
                // }
                if let Ok(decoded) =
                    self.decode(&bitmap, rotation, &expected_text, &expected_metadata, true)
                {
                    if decoded {
                        try_harder_counts[x] += 1;
                    } else {
                        try_harder_misread_counts[x] += 1;
                    }
                } else {
                    log::fine(format!("could not read at rotation {} w/TH", rotation));
                }
                // try {
                //   if (decode(bitmap, rotation, expectedText, expectedMetadata, true)) {
                //     tryHarderCounts[x]+=1;
                //   } else {
                //     tryHarderMisreadCounts[x]+=1;
                //   }
                // } catch (ReaderException ignored) {
                //   log::fine(format!("could not read at rotation {} w/TH", rotation));
                // }
            }
        }

        // Print the results of all tests first
        let mut total_found = 0;
        let mut total_must_pass = 0;
        let mut total_misread = 0;
        let mut total_max_misread = 0;

        for x in 0..self.test_rxing_results.len() {
            // for (int x = 0; x < testRXingResults.size(); x++) {
            let test_rxing_result = self.test_rxing_results.get(x).unwrap();
            log::info(format!(
                "Rotation {} degrees:",
                test_rxing_result.getRotation()
            ));
            log::info(format!(
                " {} of {} images passed ({} required)",
                passed_counts[x],
                image_files.len(),
                test_rxing_result.getMustPassCount()
            ));
            let mut failed = image_files.len() - passed_counts[x];
            log::info(format!(
                " {} failed due to misreads, {} not detected",
                misread_counts[x],
                failed - misread_counts[x]
            ));
            log::info(format!(
                " {} of {} images passed with try harder ({} required)",
                try_harder_counts[x],
                image_files.len(),
                test_rxing_result.getTryHarderCount()
            ));
            failed = image_files.len() - try_harder_counts[x];
            log::info(format!(
                " {} failed due to misreads, {} not detected",
                try_harder_misread_counts[x],
                failed - try_harder_misread_counts[x]
            ));
            total_found += passed_counts[x] + try_harder_counts[x];
            total_must_pass +=
                test_rxing_result.getMustPassCount() + test_rxing_result.getTryHarderCount();
            total_misread += misread_counts[x] + try_harder_misread_counts[x];
            total_max_misread +=
                test_rxing_result.getMaxMisreads() + test_rxing_result.getMaxTryHarderMisreads();
        }

        let totalTests = image_files.len() * test_count * 2;
        log::info(format!(
            "Decoded {} images out of {} ({}, {} required)",
            total_found,
            totalTests,
            total_found * 100 / totalTests,
            total_must_pass
        ));
        if total_found > total_must_pass as usize {
            log::warning(format!(
                "+++ Test too lax by {} images",
                total_found - total_must_pass as usize
            ));
        } else if total_found < total_must_pass as usize {
            log::warning(format!(
                "--- Test failed by {} images",
                total_must_pass as usize - total_found
            ));
        }

        if total_misread < total_max_misread as usize {
            log::warning(format!(
                "+++ Test expects too many misreads by {} images",
                total_max_misread as usize - total_misread
            ));
        } else if total_misread > total_max_misread as usize {
            log::warning(format!(
                "--- Test had too many misreads by {} images",
                total_misread - total_max_misread as usize
            ));
        }

        // Then run through again and assert if any failed
        for x in 0..test_count {
            // for (int x = 0; x < testCount; x++) {
            let testRXingResult = self.test_rxing_results.get(x).unwrap();
            let label = format!(
                "Rotation {} degrees: Too many images failed",
                testRXingResult.getRotation()
            );
            assert!(
                passed_counts[x] >= testRXingResult.getMustPassCount() as usize,
                "{}",
                label
            );
            assert!(
                try_harder_counts[x] >= testRXingResult.getTryHarderCount() as usize,
                "Try harder, {}",
                label,
            );
            let label = format!(
                "Rotation {} degrees: Too many images misread",
                testRXingResult.getRotation()
            );
            assert!(
                misread_counts[x] <= testRXingResult.getMaxMisreads() as usize,
                "{}",
                label
            );
            assert!(
                try_harder_misread_counts[x] <= testRXingResult.getMaxTryHarderMisreads() as usize,
                "Try harder, {}",
                label
            );
        }
    }

    fn decode(
        &self,
        source: &BinaryBitmap,
        rotation: f32,
        expected_text: &str,
        expected_metadata: &HashMap<RXingResultMetadataType, RXingResultMetadataValue>,
        try_harder: bool,
    ) -> Result<bool, rxing::Exceptions> {
        let suffix = format!(
            " ({}rotation: {})",
            if try_harder { "try harder, " } else { "" },
            rotation
        );

        let mut hints = self.hints.clone();
        if try_harder {
            hints.insert(DecodeHintType::TRY_HARDER, DecodeHintValue::TryHarder(true));
            // hints.put(DecodeHintType.TRY_HARDER, Boolean.TRUE);
        }

        // Try in 'pure' mode mostly to exercise PURE_BARCODE code paths for exceptions;
        // not expected to pass, generally
        let mut result = None;
        let mut pure_hints = HashMap::new();
        pure_hints.insert(
            DecodeHintType::PURE_BARCODE,
            DecodeHintValue::PureBarcode(true),
        );
        result = if let Ok(res) = self.barcode_reader.decode_with_hints(source, &pure_hints) {
            Some(res)
        } else {
            None
        };

        if result.is_none() {
            result = Some(self.barcode_reader.decode_with_hints(source, &hints)?)
        }

        let result = result.unwrap();

        if &self.expected_format != result.getBarcodeFormat() {
            log::info(format!(
                "Format mismatch: expected '{:?}' but got '{:?}'{}",
                self.expected_format,
                result.getBarcodeFormat(),
                suffix
            ));
            return Ok(false);
        }

        let result_text = result.getText();
        if !(expected_text == result_text) {
            log::info(format!(
                "Content mismatch: expected '{}' but got '{}'{}",
                expected_text, result_text, suffix
            ));
            return Ok(false);
        }

        let result_metadata = result.getRXingResultMetadata();
        for (key, value) in expected_metadata {
            // let key = RXingResultMetadataType.valueOf(metadatum.getKey().toString());
            // Object expectedValue = metadatum.getValue();
            let actual_value = result_metadata.get(key);
            if actual_value.is_none() || !(value == actual_value.unwrap()) {
                log::info(format!(
                    "Metadata mismatch for key '{:?}': expected '{:?}' but got '{:?}'",
                    key, value, actual_value
                ));
                return Ok(false);
            }
        }

        return Ok(true);
    }

    fn read_file_as_string(file: PathBuf) -> Result<String, std::io::Error> {
        let string_contents = read_to_string(&file)?; //new String(Files.readAllBytes(file), charset);
        if string_contents.ends_with("\n") {
            log::info(format!("String contents of file {} end with a newline. This may not be intended and cause a test failure",file.to_string_lossy()));
        }
        Ok(string_contents)
    }

    fn rotate_image(original: &image::DynamicImage, degrees: f32) -> image::DynamicImage {
        if degrees == 0.0 {
            return original.clone();
        }

        // switch (original.getType()) {
        //   case BufferedImage.TYPE_BYTE_INDEXED:
        //   case BufferedImage.TYPE_BYTE_BINARY:
        //     BufferedImage argb = new BufferedImage(original.getWidth(),
        //                                            original.getHeight(),
        //                                            BufferedImage.TYPE_INT_ARGB);
        //     Graphics g = argb.createGraphics();
        //     g.drawImage(original, 0, 0, null);
        //     g.dispose();
        //     original = argb;
        //     break;
        // }

        let radians = degrees.to_radians();

        {
            use image::DynamicImage;
            use image::Luma;
            use imageproc::geometric_transformations::*;

            let i = rotate_about_center(
                &original.to_luma8(),
                radians,
                Interpolation::Nearest,
                Luma([u8::MAX/2; 1]),
            );

            DynamicImage::from(i)
        }

        // // Transform simply to find out the new bounding box (don't actually run the image through it)
        // AffineTransform at = new AffineTransform();
        // at.rotate(radians, original.getWidth() / 2.0, original.getHeight() / 2.0);
        // BufferedImageOp op = new AffineTransformOp(at, AffineTransformOp.TYPE_BICUBIC);

        // RectangularShape r = op.getBounds2D(original);
        // int width = (int) Math.ceil(r.getWidth());
        // int height = (int) Math.ceil(r.getHeight());

        // // Real transform, now that we know the size of the new image and how to translate after we rotate
        // // to keep it centered
        // at = new AffineTransform();
        // at.rotate(radians, width / 2.0, height / 2.0);
        // at.translate((width - original.getWidth()) / 2.0,
        //              (height - original.getHeight()) / 2.0);
        // op = new AffineTransformOp(at, AffineTransformOp.TYPE_BICUBIC);

        // return op.filter(original, new BufferedImage(width, height, original.getType()));
    }
}

mod log {
    pub fn info(data: String) {
        prn("INFO", data)
    }

    pub fn fine(data: String) {
        prn("FINE", data)
    }

    pub fn warning(data: String) {
        prn("WARN", data)
    }

    fn prn(level: &str, data: String) {
        println!("{} :: {}", level, data)
    }
}