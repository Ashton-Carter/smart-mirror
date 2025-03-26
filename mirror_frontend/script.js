function updateTime() {
    const now = new Date();
    document.getElementById("time").innerText = now.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    document.getElementById("date").innerText = now.toLocaleDateString([], { weekday: 'long', month: 'short', day: 'numeric', year: 'numeric' });
}

async function getWeather() {
    const API_URL = "http://localhost:3000/weather?location=Orange,CA";

    try {
        const response = await fetch(API_URL);
        if (!response.ok) {
            throw new Error(`HTTP error! Status: ${response.status}`);
        }

        const data = await response.json();
        document.getElementById("weather").innerHTML = 
            `${data.location.name}, ${data.location.region} <br> Temp:${data.current.temp_f}°F <br> Condition:${data.current.condition.text}`;
            
        document.getElementById("weather-icon").src = data.current.condition.icon;
    
    } catch (error) {
        console.error("Error fetching weather data:", error);
    }
}


async function updateCalendar() {
    console.log("Called Update Calendar");
    const events = await getCalendarEvents();
    console.log("EVENTS FROM UPDATE CALENDAR:", events);
    document.getElementById("events").innerHTML = events.map(e => `<li>${e.start}:<br> ${e.title}</li>`).join("");
}

async function getCalendarEvents() {
    const API_URL = "http://localhost:3000/calendar";
    try {
        const response = await fetch(API_URL);
        if (!response.ok) {
            throw new Error(`HTTP error! Status: ${response.status}`);
        }

        const data = await response.json();
        console.log("EVENTS FROM GETCALENDAREVENTS:", data);
        return parseEvents(data);
        
    } catch (error) {
        console.error("Error fetching weather data:", error);
        return [];
    }
}

function parseEvents(events) {
    console.log("PARSING...");
    return events.map(event => {
        return {
            id: event.id,
            title: event.summary,
            description: event.description || "No description available",
            link: event.html_link,
            start: event.start.date || event.start.date_time.substring(0, 10) || "No start time",
            end: event.end.date || event.end.date_time || "No end time"
        };
    });
}

// async function sendMessage() {
//     const userInput = document.getElementById("userInput").value;
//     if (!userInput.trim()) return;

//     const API_URL = "http://localhost:3000/chat";

//     try {
//         const response = await fetch(API_URL, {
//             method: "POST",
//             headers: {
//                 "Content-Type": "application/json"
//             },
//             body: JSON.stringify({ message: userInput }) 
//         });

//         if (!response.ok) {
//             throw new Error(`HTTP error! Status: ${response.status}`);
//         }

//         const audioBlob = await response.blob(); 
//         const audioUrl = URL.createObjectURL(audioBlob); 

//         const audio = new Audio(audioUrl);
//         audio.play();

//         document.getElementById("response").innerHTML = `<strong>Playing AI Response...</strong>`;

//         document.getElementById("userInput").value = "";

//     } catch (error) {
//         console.error("Error sending message:", error);
//         document.getElementById("response").innerHTML = "Error communicating with AI.";
//     }
// }

async function executeCommand(command){
    const userInput = command;
    if (!userInput.trim()) return;

    const API_URL = "http://localhost:3000/chat"; 

    try {
        const response = await fetch(API_URL, {
            method: "POST",
            headers: {
                "Content-Type": "application/json"
            },
            body: JSON.stringify({ message: userInput })
        });

        if (!response.ok) {
            throw new Error(`HTTP error! Status: ${response.status}`);
        }

        const audioBlob = await response.blob(); 
        const audioUrl = URL.createObjectURL(audioBlob); 

        const audio = new Audio(audioUrl);
        audio.play();

        document.getElementById("response").innerHTML = `<strong>Playing AI Response...</strong>`;
        document.getElementById("userInput").value = "";
        startWakeWordRecognition();

    } catch (error) {
        console.error("Error sending message:", error);
        document.getElementById("response").innerHTML = "Error communicating with AI.";
    }
}

function activationWord () {
    const speechRecognition = window.webkitSpeechRecognition || window.SpeechRecognition;
    console.log("Running...");
    if (!speechRecognition) {
        console.log("Your browser doesn't support the Web Speech API. Please use Chrome or Edge.");
        return;
    } else {
        // Create a new instance of SpeechRecognition
        console.log("Successful activation start");
        const wakeWordRecognition = new speechRecognition();

        // Set properties
        wakeWordRecognition.continuous = true; 
        wakeWordRecognition.interimResults = false; 
        wakeWordRecognition.lang = 'en-US'; 
 

        const commandRecognition = new speechRecognition();
        commandRecognition.continuous = false;
        commandRecognition.interimResults = false;
        commandRecognition.lang = 'en-US';

        const startWakeWordRecognition = () => {
            wakeWordRecognition.start();
            console.log('Listening for the wake word...');
        };
        startWakeWordRecognition();

        
        wakeWordRecognition.onresult = (event) => {
            for (let i = event.resultIndex; i < event.results.length; i++) {
                const transcript = event.results[i][0].transcript.trim().toLowerCase();
                console.log(`You said: ${transcript}`);
                if (transcript.includes("carter")) {
                    console.log("Wake word 'carter' detected.");
                    wakeWordRecognition.stop();

                    // Provide feedback to the user
                    console.log("I'm listening for your command...");

                    // Start listening for the command
                    commandRecognition.start();
                    break;
                }
            }
        };

        // Handle command detection
        commandRecognition.onresult = (event) => {
            const command = event.results[0][0].transcript.trim();
            console.log(`Command received: ${command}`);
            // Process the command here
            executeCommand(command);
        };

        // Restart wake word recognition after command processing
        commandRecognition.onend = () => {
            console.log('Command processing ended.');
        };

        // Error handling
        wakeWordRecognition.onerror = (event) => {
            console.error('Wake word recognition error:', event.error);
            setTimeout(startWakeWordRecognition, 1000); // Retry after a short delay
        };

        commandRecognition.onerror = (event) => {
            console.error('Command recognition error:', event.error);
            alert('Sorry, I did not understand that. Please try again.');
            startWakeWordRecognition();
        };
    }
}


setInterval(updateTime, 60000);
setInterval(getWeather, 3600000);
setInterval(updateCalendar, 3600000);

activationWord();
updateTime();
getWeather();
updateCalendar();
