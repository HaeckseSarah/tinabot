function onMessage(event)
	print("Start lua2: " .. event.message)
	t.delay(5000)
	print("End lua2: " .. event.message)
end

t.on("dummy.message", {
	queue = "seq",
	filters = {},
}, onMessage)
