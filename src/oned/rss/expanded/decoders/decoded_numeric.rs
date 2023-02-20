/*
 * Copyright (C) 2010 ZXing authors
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

/*
 * These authors would like to acknowledge the Spanish Ministry of Industry,
 * Tourism and Trade, for the support in the project TSI020301-2008-2
 * "PIRAmIDE: Personalizable Interactions with Resources on AmI-enabled
 * Mobile Dynamic Environments", led by Treelogic
 * ( http://www.treelogic.com/ ):
 *
 *   http://www.piramidepse.com/
 */

use crate::common::Result;
use crate::Exceptions;

use super::DecodedObject;

/**
 * @author Pablo Orduña, University of Deusto (pablo.orduna@deusto.es)
 * @author Eduardo Castillejo, University of Deusto (eduardo.castillejo@deusto.es)
 */
pub struct DecodedNumeric {
    newPosition: usize,
    firstDigit: u32,
    secondDigit: u32,
}
impl DecodedObject for DecodedNumeric {
    fn getNewPosition(&self) -> usize {
        self.newPosition
    }
}
impl DecodedNumeric {
    pub const FNC1: u32 = 10;

    pub fn new(newPosition: usize, firstDigit: u32, secondDigit: u32) -> Result<Self> {
        // super(newPosition);

        if
        /*firstDigit < 0 ||*/
        firstDigit > 10 || /*secondDigit < 0 ||*/ secondDigit > 10 {
            return Err(Exceptions::FORMAT);
        }

        Ok(Self {
            newPosition,
            firstDigit,
            secondDigit,
        })
    }

    pub fn getFirstDigit(&self) -> u32 {
        self.firstDigit
    }

    pub fn getSecondDigit(&self) -> u32 {
        self.secondDigit
    }

    pub fn getValue(&self) -> u32 {
        self.firstDigit * 10 + self.secondDigit
    }

    pub fn isFirstDigitFNC1(&self) -> bool {
        self.firstDigit == Self::FNC1
    }

    pub fn isSecondDigitFNC1(&self) -> bool {
        self.secondDigit == Self::FNC1
    }
}
