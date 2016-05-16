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

//! Old state format. *__Will be deprecated__ when something better will become
//! available.*

use toxcore::binary_io::FromBytes;

// TODO: improve docs

/** Sections of the old state format.

https://zetok.github.io/tox-spec/#sections

## Serialization into bytes

```
use self::tox::toxcore::state_format::old_state_format::SectionKind;

assert_eq!(1u8, SectionKind::NospamKeys as u8);
assert_eq!(2u8, SectionKind::DHT as u8);
assert_eq!(3u8, SectionKind::Friends as u8);
assert_eq!(4u8, SectionKind::Name as u8);
assert_eq!(5u8, SectionKind::StatusMsg as u8);
assert_eq!(6u8, SectionKind::Status as u8);
assert_eq!(10u8, SectionKind::TcpRelays as u8);
assert_eq!(11u8, SectionKind::PathNodes as u8);
assert_eq!(255u8, SectionKind::EOF as u8);
```
*/
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SectionKind {
    /// Section for [`NoSpam`](../../toxid/struct.NoSpam.html), public and
    /// secret keys.
    NospamKeys = 0x01,
    /// Section for DHT-related data.
    DHT =        0x02,
    /// Section for friends data.
    Friends =    0x03,
    /// Section for own name.
    Name =       0x04,
    /// Section for own status message.
    StatusMsg =  0x05,
    /// Section for own status.
    Status =     0x06,
    /// Section for a list of TCP relays.
    TcpRelays =  0x0a,
    /// Section for a list of path nodes for onion routing.
    PathNodes =  0x0b,
    /// End of file.
    EOF =        0xff,
}

/** E.g.

```
use self::tox::toxcore::binary_io::FromBytes;
use self::tox::toxcore::state_format::old_state_format::SectionKind;

assert_eq!(SectionKind::NospamKeys,
        SectionKind::from_bytes(&[1]).expect("Failed to unwrap NospamKeys!"));
assert_eq!(SectionKind::DHT,
        SectionKind::from_bytes(&[2]).expect("Failed to unwrap DHT!"));
assert_eq!(SectionKind::Friends,
        SectionKind::from_bytes(&[3]).expect("Failed to unwrap Friends!"));
assert_eq!(SectionKind::Name,
        SectionKind::from_bytes(&[4]).expect("Failed to unwrap Name!"));
assert_eq!(SectionKind::StatusMsg,
        SectionKind::from_bytes(&[5]).expect("Failed to unwrap StatusMsg!"));
assert_eq!(SectionKind::Status,
        SectionKind::from_bytes(&[6]).expect("Failed to unwrap Status!"));
assert_eq!(SectionKind::TcpRelays,
        SectionKind::from_bytes(&[10]).expect("Failed to unwrap TcpRelays!"));
assert_eq!(SectionKind::PathNodes,
        SectionKind::from_bytes(&[11]).expect("Failed to unwrap PathNodes!"));
assert_eq!(SectionKind::EOF,
        SectionKind::from_bytes(&[255]).expect("Failed to unwrap EOF!"));
```
*/
// TODO: test with quickcheck
impl FromBytes<SectionKind> for SectionKind {
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        match bytes[0] {
            0x01 => Some(SectionKind::NospamKeys),
            0x02 => Some(SectionKind::DHT),
            0x03 => Some(SectionKind::Friends),
            0x04 => Some(SectionKind::Name),
            0x05 => Some(SectionKind::StatusMsg),
            0x06 => Some(SectionKind::Status),
            0x0a => Some(SectionKind::TcpRelays),
            0x0b => Some(SectionKind::PathNodes),
            0xff => Some(SectionKind::EOF),
            _ => None,
        }
    }
}