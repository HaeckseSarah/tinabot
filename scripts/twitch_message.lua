function onDummyMessage(event)
	p.twitch.send_message(string.sub(event.message, 2))
end

t.on("dummy.message", {
	queue = "seq",
	filters = { { "message", "starts_with", ">" } },
}, onDummyMessage)
