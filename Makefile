
hide-app.exe: hide-app.c
	cc -Wall -o $@ $< -Wl,--subsystem,windows
