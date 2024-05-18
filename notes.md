* Mac tools for display interactions
  * Getting / listing monitors: https://developer.apple.com/documentation/coregraphics/quartz_display_services
* Windows installer process
  * A user warned me not to rely on hardcoded paths, to be robust against localization:
    > just FYI you should never use hardcoded paths
    > use %programfiles%\Foo\foo.exe instead
  * I managed to create an appx installer, but in the process I've concluded that this is not a path I want to take
    * I have to apply for and responsibly manage a cert via Microsoft
    * Extra toil if I establish a release cadence. I'm not confident that I can automate it well.