# safe-rm

為了避免意外刪除重要檔案/目錄，可設定將重要檔案/目錄不被刪除的程式。

Copyright (C) 2008-2021 Francois Marier <francois@fmarier.org>

此程式是個自由軟體，你可以根據自由軟件基金會發佈的 GNU 通用公眾授權條款對它進行修改以及重新發佈，只要基於 GPL-3.0 或更新版本的通用公眾授權條款即可。

這個程式的發佈僅期望能對使用者有幫助，但不做任何保證。

關於通用公眾授權條款可參考[GNU 官方網站](https://www.gnu.org/licenses/)

## 環境設定

### 編譯原始碼

- 首先請先確認本專案使用的程式語言`Rust`環境是否安裝完成，可參考[Rust 官方網站](https://doc.rust-lang.org/cargo/getting-started/installation.html)
- 請在專案根目錄執行`make`，編譯成功後會產生 debug 版和 release 版，兩者的執行檔分別是`target/release/safe-rm`和`target/debug/safe-rm`，執行 debug 版會印出目前`safe-rm`排除的項目，release 版則否。

### 舊版設定方式

- 首先要將`src/main.rs`定義的原生 rm 指令所在位置參數`REAL_RM`改為原本的`/bin/rm`
- 具體設定步驟參考`INSTALL`，在各使用者家目錄下的`.bashrc`加入設定

### 新版設定方式

- 舊版設定方式有以下缺點：
  1. `sudo`指令執行指令無法讀取`.bashrc`設定
  2. 根據各設定檔引用順序，可能導致設定無法正常運作
- 改進方式為不調整現行針對 rm 指令的設定，將原生`/bin/rm`改名為`/bin/real-rm`，原始碼編譯的執行檔`target/release/safe-rm`複製到`/bin/rm`，至此所有呼叫 rm 的地方都會經過 safe-rm 的過濾。
- 在本專案根目錄執行`make install`(需管理員權限)，就能完成上述設定。

## 設定保護重要目錄

全系統範圍互通的設定檔是 /etc/safe-rm.conf 和 /usr/local/etc/safe-rm.conf，適合在這設定一些系統重要檔案，例如：

    /
    /etc
    /usr
    /usr/lib
    /var

各使用者之間不互通的設定檔是 ~/.config/safe-rm 和 ~/.safe-rm，可設定保護家目錄下的檔案，例如：

    /home/username/documents
    /home/username/documents/*
    /home/username/.mozilla

新版程式會保護重要目錄、檔案的父目錄直到系統根目錄。例如保護了`/home/user/a.txt`，連同`/home`、`/home/user`都無法刪除

## 其他方法

如果希望有更多超出 safe-rm 所能提供的保護，這裡有一些建議：

可以在 /etc/bash.bashrc 加上「alias rm='rm -i'」，使刪除任何檔案的時候都透過互動介面詢問是否刪除，但是這種設定在使用「rm -rf」參數的時候會失效。

或者使用管理員權限使用指令「chattr +i 檔名」將想保護的檔案設為"immutable"(有的檔案系統可能不支援此功能)。

以下是兩個專案，他們提供與復原近期刪除檔案相近的功能：

- [delsafe](https://web.archive.org/web/20081027033142/http://homepage.esoterica.pt:80/~nx0yew/delsafe/)
- [libtrashcan](http://hpux.connect.org.uk/hppd/hpux/Development/Libraries/libtrash-0.2/readme.html)

以下是實作了資源回收桶功能的專案：

- [trash-cli](https://github.com/andreafrancia/trash-cli)

最後的這個專案則是從 GNU 核心工具組(GNU coreutils)建立分支進行開發，透過改寫 rm 指令，直接提供與 safe-rm 相近的效果：

- [rmfd](https://github.com/d5h/rmfd)
