// Description and things
// Copyright (C) 2025 Bjarne Seger
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by the
// Free Software Foundation, version 3.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.
//
// SPDX-License-Identifier: GPL-3.0-only

pub mod common;
pub mod gui;
pub mod local;

pub const APP_ID: &str = "io.github.JustDoItBetter.fits";

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().filter_or("FITS_LOG", "warn")).init();

    gui::run();
}
