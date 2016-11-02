val windres = taskKey[File]("resource compiler")
val release = taskKey[Unit]("compile in release mode")

windres := {
  val base = baseDirectory.value
  FileFunction.cached(streams.value.cacheDirectory / "windres", FilesInfo.lastModified) { in =>
    val t = (target.value / "resource.o").getAbsolutePath
    val rc = s"windres resource.rc $t" !

    if (rc != 0) throw new MessageOnlyException("windres failed")
    Set(target.value / "resource.o")
  }(Set(base / "resource.rc", base / "resource.h")).head
}

compile in Compile := {
  val reso = windres.value.getAbsolutePath
  val rc = s"""cargo rustc -- -Clink-args="$reso"""" !

  if (rc != 0) throw new MessageOnlyException("Build failed")
  sbt.inc.Analysis.Empty
}

release := {
  val reso = windres.value.getAbsolutePath
  val rc = s"""cargo rustc --release -- -Clink-args="$reso -s -mwindows" -Cprefer-dynamic""" !

  if (rc != 0) throw new MessageOnlyException("Build failed")
}

test in Test := {
  val rc = "cargo test" !

  if (rc != 0) throw new MessageOnlyException("Tests failed")
}

run in Compile := {
  val _ = (compile in Compile).value
  val rc = """target\debug\window-hider.exe""" !

  if (rc != 0) throw new MessageOnlyException("Run failed")
}

unmanagedSources in Compile := (baseDirectory.value / "src" ** "*.rs").get
