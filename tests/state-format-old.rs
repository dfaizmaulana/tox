/*
    Copyright © 2016 Zetok Zalbavar <zexavexxe@gmail.com>

    This file is part of Tox.

    Tox is libre software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Tox is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Tox.  If not, see <http://www.gnu.org/licenses/>.
*/

extern crate tox;

use tox::toxcore::binary_io::*;
use tox::toxcore::state_format::old::*;

/*
Load bytes of a real™ profile, de-serialize it and serialize again. Serialized
again bytes must be identical, except for the zeros that trail after the data
in original implementation – they're ommited. Just check if smaller length of
the resulting bytes are in fact due to original being appended with `0`s.
*/

#[test]
fn load_old_state_format_with_contacts() {
    let bytes = include_bytes!("data/old-profile-with-contacts.tox");

    let state = match State::from_bytes(bytes) {
        IResult::Incomplete(_) => {
            panic!("no way!")
        },
        IResult::Error(e) => {
            panic!(e.description().to_string())
        },
        IResult::Done(_rest, state) => {
            state
        }
    };
    //let (_rest, profile_b) = State::from_bytes(bytes).unwrap();

    /*
    let profile_b = State::from_bytes(bytes).unwrap().to_bytes();
    assert_eq!(&bytes[..profile_b.len()], profile_b.as_slice());
    // c-toxcore appends `0`s after EOF because reasons
    for b in &bytes[profile_b.len()..] {
        assert_eq!(0, *b);
    }
    */
}

#[test]
fn load_old_state_format_no_friends() {
    let bytes = include_bytes!("data/old-profile-no-friends.tox");

    let state = match State::from_bytes(bytes) {
        IResult::Incomplete(_) => {
            panic!("no way!")
        },
        IResult::Error(e) => {
            panic!(e.description().to_string())
        },
        IResult::Done(_rest, state) => {
            state
        }
    };
}
