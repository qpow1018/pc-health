using System.Text.Json;
using LibreHardwareMonitor.Hardware;

var result = new SensorHelperOutput();
var computer = new Computer
{
    IsCpuEnabled = true,
};

try
{
    computer.Open();
    foreach (var hardware in computer.Hardware)
    {
        UpdateHardware(hardware);
        CollectSensors(hardware, result);
    }
}
catch (Exception error)
{
    result.Error = error.Message;
}
finally
{
    computer.Close();
}

Console.WriteLine(JsonSerializer.Serialize(result, JsonOptions.Default));

static void UpdateHardware(IHardware hardware)
{
    hardware.Update();
    foreach (var subHardware in hardware.SubHardware)
    {
        UpdateHardware(subHardware);
    }
}

static void CollectSensors(IHardware hardware, SensorHelperOutput result)
{
    if (hardware.HardwareType != HardwareType.Cpu)
    {
        return;
    }

    foreach (var sensor in hardware.Sensors)
    {
        if (TryGetCpuTemperature(sensor, out var value))
        {
            result.CandidateCount++;
            if (result.CpuTemperatureCelsius is null || SensorPriority(sensor) > result.Priority)
            {
                result.CpuTemperatureCelsius = value;
                result.Priority = SensorPriority(sensor);
            }
        }
    }

    foreach (var subHardware in hardware.SubHardware)
    {
        CollectSensors(subHardware, result);
    }
}

static bool TryGetCpuTemperature(ISensor sensor, out double value)
{
    value = 0;
    if (sensor.SensorType != SensorType.Temperature || sensor.Value is not { } rawValue)
    {
        return false;
    }

    value = rawValue;
    return double.IsFinite(value) && value is > 0 and <= 130;
}

static int SensorPriority(ISensor sensor)
{
    var name = sensor.Name.ToLowerInvariant();
    if (name.Contains("package") || name.Contains("tctl") || name.Contains("tdie"))
    {
        return 2;
    }

    return 1;
}

internal sealed class SensorHelperOutput
{
    public double? CpuTemperatureCelsius { get; set; }
    public int CandidateCount { get; set; }
    public string? Error { get; set; }

    internal int Priority { get; set; }
}

internal static class JsonOptions
{
    public static readonly JsonSerializerOptions Default = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
    };
}
