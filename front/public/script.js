// Exemple de données de détection
const detectionDistance = 3; // en mètres
const detectionAngle = 0.42; // en degrés

// Calculer les coordonnées du point détecté
const radius = 200; // Rayon du cercle de détection en pixels
const angleInRadians = (detectionAngle * Math.PI) / 180;
const x = radius + (radius * Math.cos(angleInRadians)) * (detectionDistance / 5);
const y = radius + (radius * Math.sin(angleInRadians)) * (detectionDistance / 5);

// Positionner le point détecté
const detectedPoint = document.getElementById('detectedPoint');
detectedPoint.style.left = `${x}px`;
detectedPoint.style.top = `${y}px`;