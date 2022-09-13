# safe-rm

prevention of accidental deletions by excluding important directories

Copyright (C) 2008-2021 Francois Marier <francois@fmarier.org>

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program. If not, see <https://www.gnu.org/licenses/>.

## How to use

Use `make` to build the source code, you will get release version and dev version (`target/release/safe-rm` and `target/debug/safe-rm`). You can execute the dev version to check which paths and files safe-rm is protecting (some system paths were added by default).

To avoid not being able to read the config in `.bashrc` when using `sudo rm` and a new user without config, I suggest using `make install`, renaming `/bin/rm` to `/bin/real-rm`, coping `target/release/safe-rm` to `/bin/rm`. This way all users will be able to avoid deleting sensitive files. Or you can edit `const REAL_RM: &str = "/bin/real-rm";` in `main.rs` to `const REAL_RM: &str = "/bin/rm";`, then check old document `INSTALL` to install.

After install safe-rm, you will need to fill the system-wide or user-specific exclusions with the paths that you'd like to protect against accidental deletion.

The system-wide exclusions live in `/etc/safe-rm.conf` (or `/usr/local/etc/safe-rm.conf`)
and you should probably add paths like these:

    /
    /etc
    /usr
    /usr/lib
    /var

The user-specific exclusions live in ~/.config/safe-rm and could include things like:

    /home/username/documents
    /home/username/documents/*
    /home/username/.mozilla

When you change setting to protect a file in a directory, safe-rm will protect its parent directory until `/`.
For example, when you add `/home/user/a.txt` into `/etc/safe-rm.conf`, the actual scope of protection will be this:

    /home/user/a.txt
    /home/user
    /home
    /

As mentioned above, you can execute the dev version safe-rm to check protected things.

## Other approaches

If you want more protection than what safe-rm can offer, here are a few suggestions.

You could of course request confirmation every time you delete a file by putting this in
your /etc/bash.bashrc:

    alias rm='rm -i'

But this won't protect you from getting used to always saying yes, or from accidentally
using 'rm -rf'.

Or you could make use of the Linux filesystem "immutable" attribute by marking (as root)
each file you want to protect:

    chattr +i file

Of course this is only usable on filesystems which support this feature.

Here are two projects which allow you to recover recently deleted files by trapping
all unlink(), rename() and open() system calls through the LD_PRELOAD facility:

- [delsafe](https://web.archive.org/web/20081027033142/http://homepage.esoterica.pt:80/~nx0yew/delsafe/)
- [libtrashcan](http://hpux.connect.org.uk/hppd/hpux/Development/Libraries/libtrash-0.2/readme.html)

There are also projects which implement the FreeDesktop.org trashcan spec. For example:

- [trash-cli](https://github.com/andreafrancia/trash-cli)

Finally, this project is a fork of GNU coreutils and adds features similar to safe-rm
to the rm command directly:

- [rmfd](https://github.com/d5h/rmfd)
