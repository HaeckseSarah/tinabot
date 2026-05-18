function onMessage(event)
	print("Hello from test2.lua")
	t.delay(1000)
	p.dummy.ping(event.message)
end

t.on("dummy.message", { { "message", "dummy.is_admin", true } }, onMessage)
