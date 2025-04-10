const eventSource = new EventSource("http://localhost:2000/sse");

// Fonction pour trouver la porte où la personne est détectée
function findGate(gates) {
    let detectedGates = [];
    gates.forEach((value, index) => {
        if (value === 100) {
            detectedGates.push(index + 1); // Les index commencent à 0, donc on ajoute 1
        }
    });

    if (detectedGates.length === 0) {
        return null; // Aucune détection
    }

    // Retourner la moyenne des portes détectées
    return detectedGates.reduce((a, b) => a + b, 0) / detectedGates.length;
}

// Fonction pour mettre à jour l'interface avec les données reçues
function updateInterface(data) {
    const state = document.getElementById('state');
    const mouvementDistanceGate0 = document.getElementById('gate0-mouvement-distance_gate');
    const mouvementDistanceGate1 = document.getElementById('gate1-mouvement-distance_gate');
    const mouvementDistanceGate2 = document.getElementById('gate2-mouvement-distance_gate');
    const mouvementDistanceGate3 = document.getElementById('gate3-mouvement-distance_gate');
    const mouvementDistanceGate4 = document.getElementById('gate4-mouvement-distance_gate');
    const mouvementDistanceGate5 = document.getElementById('gate5-mouvement-distance_gate');
    const mouvementDistanceGate6 = document.getElementById('gate6-mouvement-distance_gate');
    const mouvementDistanceGate7 = document.getElementById('gate7-mouvement-distance_gate');
    const mouvementDistanceGate8 = document.getElementById('gate8-mouvement-distance_gate');
    const staticDistanceGate0 = document.getElementById('gate0-static-distance_gate');
    const staticDistanceGate1 = document.getElementById('gate1-static-distance_gate');
    const staticDistanceGate2 = document.getElementById('gate2-static-distance_gate');
    const staticDistanceGate3 = document.getElementById('gate3-static-distance_gate');
    const staticDistanceGate4 = document.getElementById('gate4-static-distance_gate');
    const staticDistanceGate5 = document.getElementById('gate5-static-distance_gate');
    const staticDistanceGate6 = document.getElementById('gate6-static-distance_gate');
    const staticDistanceGate7 = document.getElementById('gate7-static-distance_gate');
    const staticDistanceGate8 = document.getElementById('gate8-static-distance_gate');
    const movementTargetDistance = document.getElementById('movement_target_distance');
    const staticTargetDistance = document.getElementById('stationary_target_distance');
    const detectionTargetDistance = document.getElementById('detection_distance');

    const movementGates = data.target_data.engineering_model?.mouvement_distance_gates;
    const staticGates = data.target_data.engineering_model?.static_distance_gates;

    mouvementDistanceGate0.innerHTML = data.target_data.engineering_model?.mouvement_distance_gates[0];
    mouvementDistanceGate1.innerHTML = data.target_data.engineering_model?.mouvement_distance_gates[1];
    mouvementDistanceGate2.innerHTML = data.target_data.engineering_model?.mouvement_distance_gates[2];
    mouvementDistanceGate3.innerHTML = data.target_data.engineering_model?.mouvement_distance_gates[3];
    mouvementDistanceGate4.innerHTML = data.target_data.engineering_model?.mouvement_distance_gates[4];
    mouvementDistanceGate5.innerHTML = data.target_data.engineering_model?.mouvement_distance_gates[5];
    mouvementDistanceGate6.innerHTML = data.target_data.engineering_model?.mouvement_distance_gates[6];
    mouvementDistanceGate7.innerHTML = data.target_data.engineering_model?.mouvement_distance_gates[7];
    mouvementDistanceGate8.innerHTML = data.target_data.engineering_model?.mouvement_distance_gates[8];
    staticDistanceGate0.innerHTML = data.target_data.engineering_model?.static_distance_gates[0];
    staticDistanceGate1.innerHTML = data.target_data.engineering_model?.static_distance_gates[1];
    staticDistanceGate2.innerHTML = data.target_data.engineering_model?.static_distance_gates[2];
    staticDistanceGate3.innerHTML = data.target_data.engineering_model?.static_distance_gates[3];
    staticDistanceGate4.innerHTML = data.target_data.engineering_model?.static_distance_gates[4];
    staticDistanceGate5.innerHTML = data.target_data.engineering_model?.static_distance_gates[5];
    staticDistanceGate6.innerHTML = data.target_data.engineering_model?.static_distance_gates[6];
    staticDistanceGate7.innerHTML = data.target_data.engineering_model?.static_distance_gates[7];
    staticDistanceGate8.innerHTML = data.target_data.engineering_model?.static_distance_gates[8];

    staticTargetDistance.innerHTML = data.target_data.stationary_target_distance / 100;
    detectionTargetDistance.innerHTML = data.target_data.detection_distance / 100;
    movementTargetDistance.innerHTML = data.target_data.movement_target_distance / 100;

    // Trouver la porte où la personne bouge ou est statique
    const movementGate = findGate(movementGates);
    const staticGate = findGate(staticGates);

    // Déterminer la porte principale (priorité au mouvement)
    const detectedGate = movementGate || staticGate;

    if (detectedGate === null) {
        state.innerHTML = "Aucune détection";
        console.log("Aucune détection");
        return;
    }

    // Calculer la distance en fonction de la porte détectée
    const detectionDistance = data.target_data.detection_distance; // 0.75m par porte

    // Calculer l'angle de détection
    const angleInRadians = Math.atan((detectionDistance / 100) / (0.75 * detectedGate));
    const detectionAngle = (angleInRadians * 180) / Math.PI; // Convertir en degrés


    // Afficher les informations dans la console
    state.innerHTML = "Porte détectée : " + detectedGate;
    distance.innerHTML = detectionDistance / 100;
    angle.innerHTML = detectionAngle;
    console.log(`Porte détectée : ${detectedGate}`);
    console.log(`Distance : ${detectionDistance.toFixed(2)} m`);
    console.log(`Angle : ${detectionAngle.toFixed(2)}°`);
}

eventSource.onmessage = (event) => {
    const data = JSON.parse(event.data); // Convertir les données JSON
    console.log("Received data:", data);
    updateInterface(data); // Mettre à jour l'interface avec les données reçues
};

eventSource.onerror = (error) => {
    console.error("Error with SSE:", error);
};